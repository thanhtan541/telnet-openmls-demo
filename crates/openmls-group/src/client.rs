use std::{io, net::SocketAddr};

use futures::stream::StreamExt;
use tokio::{
    io::AsyncWriteExt,
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
    select,
    sync::{
        mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
    try_join,
};
use tokio_util::codec::FramedRead;

use crate::{
    main_loop::{ServerHandle, ToDelivery},
    telnet::{Item, TelnetCodec},
    ClientId,
};

/// Messages received from the main loop.
pub enum FromDelivery {
    // Should be decrypted data
    Message(Vec<u8>),
}

/// This struct is constructed by the accept loop and used as the argument to
/// `spawn_client`.
pub struct ClientInfo {
    pub id: ClientId,
    pub ip: SocketAddr,
    pub handle: ServerHandle,
    pub tcp: TcpStream,
}

struct ClientData {
    id: ClientId,
    handle: ServerHandle,
    recv: Receiver<FromDelivery>,
    tcp: TcpStream,
}

/// A handle to this actor, used by the server.
#[derive(Debug)]
pub struct ClientHandle {
    pub id: ClientId,
    ip: SocketAddr,
    chan: Sender<FromDelivery>,
    kill: JoinHandle<()>,
}

impl ClientHandle {
    pub fn send(&mut self, msg: FromDelivery) -> Result<(), io::Error> {
        if self.chan.try_send(msg).is_err() {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Can't keep up or dead",
            ))
        } else {
            Ok(())
        }
    }
    /// Kill the actor.
    pub fn kill(self) {
        // run the destructor
        drop(self);
    }
}

impl Drop for ClientHandle {
    fn drop(&mut self) {
        self.kill.abort()
    }
}

pub fn spawn_client(info: ClientInfo) {
    let (send, recv) = channel(64);

    let data = ClientData {
        id: info.id,
        handle: info.handle.clone(),
        tcp: info.tcp,
        recv,
    };

    // This spawns the new task.
    let (my_send, my_recv) = oneshot::channel();
    let kill = tokio::spawn(start_client(my_recv, data));

    // Then we create a ClientHandle to this new task, and use the oneshot
    // channel to send it to the task.
    let handle = ClientHandle {
        id: info.id,
        ip: info.ip,
        chan: send,
        kill,
    };

    // Ignore send errors here. Should only happen if the server is shutting
    // down.
    let _ = my_send.send(handle);
}

async fn start_client(my_handle: oneshot::Receiver<ClientHandle>, mut data: ClientData) {
    // Wait for `spawn_client` to send us the `ClientHandle` so we can forward
    // it to the main loop. We need the oneshot channel because we cannot
    // otherwise get the `JoinHandle` returned by `tokio::spawn`. We forward it
    // from here instead of in `spawn_client` because we want the server to see
    // the NewClient message before this actor starts sending other messages.
    let my_handle = match my_handle.await {
        Ok(my_handle) => my_handle,
        Err(_) => return,
    };
    data.handle.send(ToDelivery::NewClient(my_handle)).await;

    // We sent the client handle to the main loop. Start talking to the tcp
    // connection.
    let res = client_loop(data).await;
    match res {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Something went wrong: {}.", err);
        }
    }
}

/// This method performs the actual job of running the client actor.
async fn client_loop(mut data: ClientData) -> Result<(), io::Error> {
    let (read, write) = data.tcp.split();

    // communication between tcp_read and tcp_write
    let (send, recv) = unbounded_channel();

    let ((), ()) = try_join! {
        tcp_read(data.id, read, data.handle, send),
        tcp_write(write, data.recv, recv),
    }?;

    let _ = data.tcp.shutdown().await;

    Ok(())
}

#[derive(Debug)]
enum InternalMsg {
    GotAreYouThere,
    SendDont(u8),
    SendWont(u8),
    SendDo(u8),
}

async fn tcp_read(
    id: ClientId,
    read: ReadHalf<'_>,
    mut handle: ServerHandle,
    to_tcp_write: UnboundedSender<InternalMsg>,
) -> Result<(), io::Error> {
    let mut telnet = FramedRead::new(read, TelnetCodec::new());

    while let Some(item) = telnet.next().await {
        match item? {
            Item::AreYouThere => {
                to_tcp_write
                    .send(InternalMsg::GotAreYouThere)
                    .expect("Should not be closed.");
            }
            Item::GoAhead => { /* ignore */ }
            Item::InterruptProcess => return Ok(()),
            Item::Will(3) => {
                // suppress go-ahead
                to_tcp_write
                    .send(InternalMsg::SendDo(3))
                    .expect("Should not be closed.");
            }
            Item::Will(i) => {
                to_tcp_write
                    .send(InternalMsg::SendDont(i))
                    .expect("Should not be closed.");
            }
            Item::Do(i) => {
                to_tcp_write
                    .send(InternalMsg::SendWont(i))
                    .expect("Should not be closed.");
            }
            Item::Line(line) => {
                handle.send(ToDelivery::Message(id, line)).await;
            }
            Item::ShowKPDetails => {
                println!("Todo");
            }
            Item::PublishKeyPackage => {
                println!("[Client] Publishing key package of client: {}", id);
            }
            item => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unable to handle {:?}", item),
                ));
            }
        }
    }

    // disconnected

    Ok(())
}

async fn tcp_write(
    mut write: WriteHalf<'_>,
    mut recv: Receiver<FromDelivery>,
    mut from_tcp_read: UnboundedReceiver<InternalMsg>,
) -> Result<(), io::Error> {
    loop {
        select! {
            msg = recv.recv() => match msg {
                Some(FromDelivery::Message(msg)) => {
                    write.write_all(&msg).await?;
                    write.write_all(&[13, 10]).await?;
                },
                None => {
                    break;
                },
            },
            msg = from_tcp_read.recv() => match msg {
                Some(InternalMsg::GotAreYouThere) => {
                    write.write_all(b"Yes.\r\n").await?;
                },
                Some(InternalMsg::SendDont(i)) => {
                    write.write_all(&[0xff, 254, i]).await?;
                },
                Some(InternalMsg::SendWont(i)) => {
                    write.write_all(&[0xff, 252, i]).await?;
                },
                Some(InternalMsg::SendDo(i)) => {
                    write.write_all(&[0xff, 253, i]).await?;
                },
                None => {
                    break;
                },
            },
        };
    }

    Ok(())
}
