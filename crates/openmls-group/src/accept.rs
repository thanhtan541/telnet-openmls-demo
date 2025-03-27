use std::io;
use std::net::SocketAddr;

use crate::client::{spawn_client, ClientInfo};
use crate::main_loop::{ServerHandle, ToDelivery};

use tokio::net::TcpListener;

pub async fn start_accept(bind: SocketAddr, mut handle: ServerHandle) {
    let res = accept_loop(bind, handle.clone()).await;
    match res {
        Ok(()) => {}
        Err(err) => {
            handle.send(ToDelivery::FatalError(err)).await;
        }
    }
}

pub async fn accept_loop(bind: SocketAddr, handle: ServerHandle) -> Result<(), io::Error> {
    let listen = TcpListener::bind(bind).await?;

    loop {
        let (tcp, ip) = listen.accept().await?;
        println!("[Client] tcp: {:?}", tcp);
        println!("[Client] ip: {:?}", ip);

        let id = handle.next_id();

        let data = ClientInfo {
            ip,
            id,
            tcp,
            handle: handle.clone(),
        };

        spawn_client(data);
    }
}
