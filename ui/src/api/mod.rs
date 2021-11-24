use seed::prelude::{fetch, Method, Request};

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

pub async fn post(url: String, bytes: Vec<u8>) -> fetch::Result<Vec<u8>> {
    send_request(Method::Post, url, bytes).await
}

pub async fn get(url: String) -> fetch::Result<Vec<u8>> {
    Request::new(url.as_str())
        .method(Method::Get)
        .fetch()
        .await?
        .check_status()?
        .bytes()
        .await
}
