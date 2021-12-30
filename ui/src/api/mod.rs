use seed::prelude::{fetch, Method, Request};
use shared::api::endpoint::Endpoint;

async fn send_request(method: Method, url: String, bytes: Vec<u8>) -> fetch::Result<Vec<u8>> {
    Request::new(url.as_str())
        .method(method)
        .text(hex::encode(bytes))
        .fetch()
        .await?
        .check_status()?
        .bytes()
        .await
}

pub async fn post(endpoint: Endpoint, bytes: Vec<u8>) -> fetch::Result<Vec<u8>> {
    send_request(Method::Post, endpoint.to_string(), bytes).await
}

pub async fn get(endpoint: Endpoint) -> fetch::Result<Vec<u8>> {
    Request::new(endpoint.to_string().as_str())
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .bytes()
        .await
}
