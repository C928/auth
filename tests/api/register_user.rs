use crate::utils::start_test_server;

#[actix_web::test]
async fn register_user() {
    let utils = start_test_server().await;
    let res = utils
        .http_client
        .get("https://127.0.0.1:8443/register")
        .send()
        .await;
    assert!(res.is_ok());
}
