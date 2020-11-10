use std::net::IpAddr;

use crate::tcp_client::TcpClient;
use std::time::Duration;
use crate::{rpc, Error};
use crate::rpc::{Serialize, Deserialize};
use xdr_rs_serialize::ser::XDROut;
use xdr_rs_serialize::de::XDRIn;

const PROG: u32 = 0x0607af;
const VERS: u32 = 1;

const CALL_CREATE_LINK: u32 = 10;
const CALL_DESTROY_LINK: u32 = 23;

struct CreateLinkRequest {
    client_id: i32,
    lock: bool,
    lock_timeout_ms: u32,
    device: String,
}

impl Serialize for CreateLinkRequest {
    fn serialize(&self, out: &mut Vec<u8>) {
        self.client_id.write_xdr(out).unwrap();
        self.lock.write_xdr(out).unwrap();
        self.lock_timeout_ms.write_xdr(out).unwrap();
        self.device.write_xdr(out).unwrap();
    }
}

struct CreateLinkResponse {
    error: u32,
    link_id: u32,
    port: u32,
    max_recv_size: u32,
}

impl Deserialize for CreateLinkResponse {
    fn deserialize_partial(data: &[u8]) -> crate::Result<(Self, usize)> {
        let (error, offset) = u32::read_xdr(&data[0..]).map_err(Error::XdrError)?;
        let (link_id, offset) = u32::read_xdr(&data[offset as usize..]).map_err(Error::XdrError)?;
        let (port, offset) = u32::read_xdr(&data[offset as usize..]).map_err(Error::XdrError)?;
        let (max_recv_size, offset) = u32::read_xdr(&data[offset as usize..]).map_err(Error::XdrError)?;

        let ret = CreateLinkResponse {
            error, link_id, port, max_recv_size
        };
        Ok((ret, offset as usize))
    }
}

struct CoreClient {
    client: TcpClient,
    abort_port: Option<u16>,
}

impl CoreClient {
    pub async fn connect<T: Into<IpAddr>>(addr: T) -> crate::Result<Self> {
        let client = TcpClient::connect_with_mapper(addr, PROG, VERS).await?;
        let mut ret = Self {
            client,
            abort_port: None
        };

        ret.create_link(false, Duration::from_secs(0)).await?;
        Ok(ret)
    }

    pub async fn create_link(&mut self, lock: bool, lock_timeout: Duration) -> crate::Result<()> {
        let req = CreateLinkRequest {
            client_id: 1,
            lock,
            lock_timeout_ms: lock_timeout.as_millis() as u32,
            device: "instr".to_string()
        };
        let resp: CreateLinkResponse = rpc::call(&mut self.client, &req, PROG, VERS, CALL_CREATE_LINK).await?;
        if resp.port < 65535 {
            self.abort_port = Some(resp.port as u16);
        } else {
            return Err(Error::InvalidPortNumber);
        }
        if resp.error != 0 {
            Err(Error::VxiRemoteError(resp.error))
        } else {
            Ok(())
        }
    }

    pub async fn destroy_link(&mut self) -> crate::Result<()> {
        let client_id = 1_u32;
        let err: u32 = rpc::call(&mut self.client, &client_id, PROG, VERS, CALL_DESTROY_LINK).await?;
        if err != 0 {
            Err(Error::VxiRemoteError(err))
        } else {
            Ok(())
        }
    }

}
