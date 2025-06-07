use actix_cors::Cors;
use actix_web::{dev::Server, web::Data, App, HttpServer};
use std::{io::Error, net::TcpListener};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::Settings,
    routes::{health_check, index, qr},
};

pub struct ApplicationBaseUrl(pub String);
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).expect(&format!(
            "Failed to bind port {}",
            configuration.application.port
        ));
        let port = listener.local_addr().unwrap().port();

        let server = run(listener, configuration.application.base_url).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server.await
    }
}

async fn run(listener: TcpListener, base_url: String) -> Result<Server, anyhow::Error> {
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            // .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            // Logger middleware
            // Sent active-web log to log subscriber
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(index)
            .service(health_check)
            .service(qr)
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
