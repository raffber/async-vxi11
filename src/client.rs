use onc_rpc::{CallBody, AcceptedReply};
use async_trait::async_trait;
use bytes::Bytes;


pub type Request = CallBody<Vec<u8>, Vec<u8>>;

#[async_trait]
pub trait Client {
    async fn call(&mut self, body: Request) -> crate::Result<Bytes>;
}


