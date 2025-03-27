use openmls::prelude::KeyPackageBundle;
use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::{
    client::{ClientHandle, FromDelivery},
    ClientId,
};

// Define the messages the actor can handle
pub enum ToDelivery {
    NewClient(ClientHandle),
    Message(ClientId, Vec<u8>),
    KeyPackage(ClientId, KeyPackageBundle),
    FetchKeyPackage(ClientId, Vec<ClientId>),
    FatalError(io::Error),
}

/// This struct is used by client actors to send messages to the main loop. The
/// message type is `ToDelivery`.
#[derive(Clone, Debug)]
pub struct ServerHandle {
    chan: Sender<ToDelivery>,
    next_id: Arc<AtomicUsize>,
}

impl ServerHandle {
    pub async fn send(&mut self, msg: ToDelivery) {
        if self.chan.send(msg).await.is_err() {
            panic!("Main loop has shut down.");
        }
    }

    pub fn next_id(&self) -> ClientId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        ClientId(id)
    }
}

#[derive(Default, Debug)]
struct Data {
    clients: HashMap<ClientId, ClientHandle>,
}

// Delivery Service store
#[derive(Default, Debug)]
pub struct DSStore {
    key_packages: HashMap<usize, Vec<KeyPackageBundle>>,
}

impl DSStore {
    pub fn add_key_package(&mut self, id: usize, value: KeyPackageBundle) {
        self.key_packages
            .entry(id)
            .or_insert_with(Vec::new)
            .push(value);
    }
}

pub fn spawn_main_loop() -> (ServerHandle, JoinHandle<()>) {
    let (send, recv) = channel(64);

    let handle = ServerHandle {
        chan: send,
        next_id: Default::default(),
    };

    let join = tokio::spawn(async move {
        let res = main_loop(recv).await;
        match res {
            Ok(()) => {}
            Err(err) => {
                eprintln!("Oops {}.", err);
            }
        }
    });

    (handle, join)
}

async fn main_loop(mut recv: Receiver<ToDelivery>) -> Result<(), io::Error> {
    let mut data = Data::default();
    let mut ds_store = DSStore::default();

    while let Some(msg) = recv.recv().await {
        match msg {
            ToDelivery::NewClient(handle) => {
                println!("[Delivery Service] received new client");
                data.clients.insert(handle.id, handle);

                let otp = "1234";
                println!("[Delivery Service] generated new OTP: {}", otp);
                println!("[Delivery Service] sent OTP to client");
                let msg_to_client = "Please provide the otp!";
                let msg = FromDelivery::Message(msg_to_client.as_bytes().to_vec());

                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == handle.id {
                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };

                        break;
                    }
                }
            }
            ToDelivery::Message(from_id, msg) => {
                // If we fail to send messages to any actor, we need to remove
                // it, but we can't do so while iterating.
                // let mut to_remove = Vec::new();

                println!("[Delivery Service] received message");
                // Iterate through clients so we can send the message.
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Don't send it to the client who sent it to us.
                    if id == from_id {
                        continue;
                    }

                    let msg = FromDelivery::Message(msg.clone());

                    match handle.send(msg) {
                        Ok(()) => {}
                        Err(err) => {
                            eprintln!("[Delivery Service] Something went wrong: {}.", err);
                        }
                    };
                }
            }
            ToDelivery::FetchKeyPackage(from_id, ids) => {
                println!(
                    "[Delivery Service] received request to fetch key package of ids: {:?}",
                    ids
                );
                let mut kps: HashMap<ClientId, KeyPackageBundle> = HashMap::new();
                for id in ids {
                    if let Some(key_packages) = ds_store.key_packages.get(&id.0).as_mut() {
                        let key_package = key_packages
                            .clone()
                            .pop()
                            .expect("Failed to get key package");
                        kps.insert(id, key_package);
                    }
                }
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Notify to the client who sent it to us.
                    if id == from_id {
                        let msg = FromDelivery::SendKeyPackages(kps.clone());
                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };
                    }
                }
            }
            ToDelivery::KeyPackage(from_id, key_package_bundle) => {
                println!("[Delivery Service] received key package of: {:?}", from_id);
                ds_store.add_key_package(from_id.0, key_package_bundle);
                println!(
                    "[Delivery Service] Total key packages is: {}",
                    ds_store.key_packages.get(&from_id.0).unwrap().len()
                );
                // Iterate through clients so we can send the message.
                for (id, handle) in data.clients.iter_mut() {
                    let id = *id;

                    // Notify to the client who sent it to us.
                    if id == from_id {
                        let msg =
                            FromDelivery::Message(Vec::from("[Server] Key package received!"));
                        match handle.send(msg) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("[Delivery Service] Something went wrong: {}.", err);
                            }
                        };
                    }
                }
            }
            ToDelivery::FatalError(err) => return Err(err),
        }
    }

    Ok(())
}
