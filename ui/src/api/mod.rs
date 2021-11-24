use seed::prelude::{fetch, Method, Request};

async fn send_request(url: String, bytes: Vec<u8>) -> fetch::Result<Vec<u8>> {
    Request::new(url.as_str())
        .method(Method::Post)
        .text(hex::encode(bytes))
        .fetch()
        .await?
        .check_status()?
        .bytes()
        .await
}

pub async fn post(url: String, bytes: Vec<u8>) -> fetch::Result<Vec<u8>> {
    send_request(url, bytes).await
}
