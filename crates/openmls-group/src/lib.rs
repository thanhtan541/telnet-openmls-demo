// Spawn actors for one server and multiple clients
// Server will have one tx_server will be shared among clients
// Server will have all client's rx_clients

// Assumption:
// Server could be main thread
// Client will be spawned thread
pub mod accept;
pub mod client;
pub mod main_loop;
pub mod telnet;

use std::fmt::Display;

use main_loop::ToDelivery;
use rand::Rng;
use tokio::sync::{
    mpsc::{channel, Receiver},
    oneshot,
};

enum ToIdentity {
    // Authentication
    GetOTP {
        resp: oneshot::Sender<i32>,
    }, // Get the current count
    SubmitOTP {
        data: i32,
        resp: oneshot::Sender<i32>,
    }, // Get the current count
       // Public key management
}

struct IdentityActor {}

impl IdentityActor {
    pub fn new() -> Self {
        Self {}
    }

    async fn run(self, mut rx: Receiver<ToIdentity>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                ToIdentity::GetOTP { resp } => {
                    let otp = generate_otp(6);
                    println!("[Identity Service] generated otp: {otp}");
                    let _ = resp.send(otp); // Send the count back via oneshot channel
                }
                ToIdentity::SubmitOTP { data, resp } => {
                    println!("[Identity Service] received otp: {data}");
                    let token = data;
                    println!("[Identity Service] generated token: {token}");
                    let _ = resp.send(token); // Send the key back via oneshot channel
                }
            }
        }
    }
}

fn generate_otp(length: u32) -> i32 {
    let lower_bound = 10i32.pow(length - 1);
    let upper_bound = 10i32.pow(length) - 1;

    // Generate a random number in the range [lower_bound, upper_bound]
    let mut rng = rand::thread_rng();
    rng.gen_range(lower_bound..=upper_bound)
}

struct DeliveryActor;

impl DeliveryActor {
    pub fn new() -> Self {
        Self {}
    }

    async fn run(self, mut rx: Receiver<ToDelivery>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                ToDelivery::Message(client_id, data) => {
                    println!(
                        "[Delivery] received message: {:?}, from client {}",
                        data, client_id.0
                    );
                }
                _ => {}
            }
        }
    }
}

pub async fn main_otp_loop() {
    let (tx, rx) = channel(100);

    tokio::spawn(async move {
        let identity_service = IdentityActor::new();
        identity_service.run(rx).await;
    });

    for _ in 1..10 {
        // Client
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(ToIdentity::GetOTP { resp: resp_tx }).await.unwrap();
        let otp = resp_rx.await.unwrap();
        println!("[Client] Current otp is: {otp}");

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(ToIdentity::SubmitOTP {
            data: otp,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let token = resp_rx.await.unwrap();
        println!("[Client] Current token is: {token}");
    }
}

pub async fn main_message_loop() {
    let (tx, rx) = channel(100);

    tokio::spawn(async move {
        let delivery_service = DeliveryActor::new();
        delivery_service.run(rx).await;
    });

    for n in 1..10 {
        // Client
        let client_id = ClientId(n);
        tx.send(ToDelivery::Message(client_id, "hello from client".into()))
            .await
            .unwrap();
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClientId(pub usize);

impl Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client({})", self.0)
    }
}
