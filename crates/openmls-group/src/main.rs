use openmls_group::{accept::start_accept, main_loop::spawn_main_loop};

#[tokio::main]
async fn main() {
    let (handle, join) = spawn_main_loop();
    let port = 3456;

    tokio::spawn(async move {
        let bind = ([0, 0, 0, 0], port.clone()).into();
        start_accept(bind, handle).await;
    });

    println!("[Server] Starting on port {}", port);
    println!("[Server] Use:");
    println!("[Server]      telnet 127.0.0.1 {}", port);
    println!("[Server] to connect.");

    join.await.unwrap();
}
