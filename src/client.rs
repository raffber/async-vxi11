use onc_rpc::CallBody;
use async_trait::async_trait;
use bytes::Bytes;
use onc_rpc::auth::AuthFlavor;


pub type Request = CallBody<Vec<u8>, Vec<u8>>;

#[async_trait]
pub trait Client {
    async fn call(&mut self, body: Request) -> crate::Result<Bytes>;

    fn make_request(&self, prog: u32, vers: u32, proc: u32, payload: Vec<u8>) -> Request {
        CallBody::new(prog, vers, proc,
                                    AuthFlavor::AuthNone(None),
                                    AuthFlavor::AuthNone(None), payload)
    }
}


