use std::net::IpAddr;

use async_trait::async_trait;
use bytes::Bytes;
use onc_rpc::auth::AuthFlavor;
use onc_rpc::CallBody;
use xdr_rs_serialize::de::XDRIn;
use xdr_rs_serialize::ser::XDROut;

pub type Request = CallBody<Vec<u8>, Vec<u8>>;

/// Trait defining the transport layer over which the VXI-11 protocol runs.
/// This trait uses `#[async_trait]` - Its "actual" signature is as follows:
///
/// ```
/// #[async_trait]
/// pub trait Client: Sized {
///     async fn connect_with_mapper<T: Into<IpAddr> + Send>(
///        addr: T,
///        prog: u32,
///        vers: u32,
///    ) -> crate::Result<Self>;
///    async fn call(&mut self, body: Request) -> crate::Result<Bytes>;
///}
/// ```
#[async_trait]
pub trait Client: Sized {
    /// Connect the the server with the portmapper protocol.
    async fn connect_with_mapper<T: Into<IpAddr> + Send>(
        addr: T,
        prog: u32,
        vers: u32,
    ) -> crate::Result<Self>;

    /// Perform an RPC call and return the reply
    async fn call(&mut self, body: Request) -> crate::Result<Bytes>;
}

/// Trait for XDR serialization. It contains a blanked impl for all types
/// implementing [`XDROut`]
pub trait Serialize {
    fn serialize(&self, out: &mut Vec<u8>);
}

impl<T: XDROut> Serialize for T {
    fn serialize(&self, out: &mut Vec<u8>) {
        self.write_xdr(out).unwrap();
    }
}

/// Trait for XDR deserialization. It contains a blanked impl for all types
/// implementing [`XDRIn`]
pub trait Deserialize
where
    Self: Sized,
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

/// Perform an RPC call with the given client and the given request and
/// reponse types..
pub async fn call<C: Client, Req: Serialize, Resp: Deserialize>(
    client: &mut C,
    req: &Req,
    prog: u32,
    vers: u32,
    call: u32,
) -> crate::Result<Resp> {
    let mut payload = Vec::new();
    req.serialize(&mut payload);
    let req = CallBody::new(
        prog,
        vers,
        call,
        AuthFlavor::AuthNone(None),
        AuthFlavor::AuthNone(None),
        payload,
    );
    log::debug!("Initiating call from prog={}, call={}", prog, call);
    let data = client.call(req).await?;
    log::debug!("Got response with length: {}", data.len());
    Resp::deserialize(&data)
}
