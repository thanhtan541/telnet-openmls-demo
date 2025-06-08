use fake::faker::internet::en::SafeEmail;
use fake::Fake;
use once_cell::sync::Lazy;
use uuid::Uuid;
use web::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub test_user: TestUser,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: SafeEmail().fake(),
            password: Uuid::new_v4().to_string(),
        }
    }
}

impl TestApp {
    pub async fn get_healthcheck(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/health_check", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    // Singleton Pattern
    Lazy::force(&TRACING);

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        // Wildcard port, the system will find available port
        c.application.port = 0;
        c
    };
    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);

    // Run the application
    let _ = tokio::spawn(app.run_until_stopped());
    let test_app = TestApp {
        address,
        port,
        api_client,
        test_user: TestUser::generate(),
    };
    // Add test user
    test_app
}
