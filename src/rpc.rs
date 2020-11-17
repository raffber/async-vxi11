use async_trait::async_trait;
use bytes::Bytes;
use onc_rpc::auth::AuthFlavor;
use onc_rpc::CallBody;
use xdr_rs_serialize::de::XDRIn;
use xdr_rs_serialize::ser::XDROut;

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

pub trait Serialize {
    fn serialize(&self, out: &mut Vec<u8>);
}

impl<T: XDROut> Serialize for T {
    fn serialize(&self, out: &mut Vec<u8>) {
        self.write_xdr(out).unwrap();
    }
}

pub trait Deserialize
    where
        Self: Sized
{
    fn deserialize(data: &[u8]) -> crate::Result<Self> {
        let (ret, _) = Self::deserialize_partial(data)?;
        Ok(ret)
    }

    fn deserialize_partial(data: &[u8]) -> crate::Result<(Self, usize)>;
}

impl<T: XDRIn> Deserialize for T {
    fn deserialize_partial(data: &[u8]) -> crate::Result<(Self, usize)> {
        let (ret, len) = T::read_xdr(data).map_err(crate::Error::XdrError)?;
        Ok((ret, len as usize))
    }
}

pub async fn call<C: Client, Req: Serialize, Resp: Deserialize>(client: &mut C, req: &Req, prog: u32, vers: u32, call: u32) -> crate::Result<Resp> {
    let mut payload = Vec::new();
    req.serialize(&mut payload);
    let req = client.make_request(prog, vers, call, payload);
    log::debug!("Initiating call from prog={}, call={}", prog, call);
    let data = client.call(req).await?;
    log::debug!("Got response with length: {}", data.len());
    Resp::deserialize(&data)
}
