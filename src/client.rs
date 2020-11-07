use onc_rpc::{CallBody, AcceptedReply};
use async_trait::async_trait;


pub type Request = CallBody<Vec<u8>, Vec<u8>>;
pub type Reply = AcceptedReply<Vec<u8>, Vec<u8>>;

#[async_trait]
pub trait Client {
    async fn call(&mut self, body: Request) -> crate::Result<Reply>;
}


