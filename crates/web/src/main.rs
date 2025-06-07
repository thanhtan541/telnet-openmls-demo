use web::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    // Redirect all `log`'s event to our subscriber
    let subscriber = get_subscriber("web".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let app = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(app.run_until_stopped());

    tokio::select! {
        _ = application_task => {}
    };

    Ok(())
}
