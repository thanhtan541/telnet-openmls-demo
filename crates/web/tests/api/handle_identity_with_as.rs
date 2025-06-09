use crate::helpers::spawn_app;

#[tokio::test]
async fn post_identities_works() {
    let app = spawn_app().await;

    let response = app.post_identities().await;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
