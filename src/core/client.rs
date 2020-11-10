use std::net::IpAddr;

use crate::tcp_client::TcpClient;
use std::time::Duration;
use crate::{rpc, Error};
use crate::rpc::{Serialize, Deserialize};
use xdr_rs_serialize::ser::XDROut;
use xdr_rs_serialize::de::XDRIn;
use crate::core::calls::{CreateLinkRequest, CreateLinkResponse, DeviceWriteRequest, DeviceWriteResponse};

const PROG: u32 = 0x0607af;
const VERS: u32 = 1;

const CALL_CREATE_LINK: u32 = 10;
const CALL_DESTROY_LINK: u32 = 23;
const CALL_DEVICE_WRITE: u32 = 11;

const IO_TIMEOUT_MS: u64 = 1000;

#[derive(Clone)]
struct VxiOptions {
    termchr: u8,
    lock_timeout: Duration,
    io_timeout: Duration,
}

impl Default for VxiOptions {
    fn default() -> Self {
        Self {
            termchr: 10, // \n
            lock_timeout: Default::default(),
            io_timeout: Duration::from_millis(IO_TIMEOUT_MS)
        }
    }
}

struct CoreClient {
    client: TcpClient,
    abort_port: Option<u16>,
    options: VxiOptions,
}

impl CoreClient {
    pub async fn connect<T: Into<IpAddr>>(addr: T) -> crate::Result<Self> {
        let client = TcpClient::connect_with_mapper(addr, PROG, VERS).await?;
        let mut ret = Self {
            client,
            abort_port: None,
            options: Default::default()
        };

        ret.create_link(false, Default::default()).await?;
        Ok(ret)
    }

    pub fn adjust_options(&mut self, options: VxiOptions) {
        self.options = options;
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

    pub async fn device_write(&mut self, data: Vec<u8>) -> crate::Result<usize> {
        let request = DeviceWriteRequest {
            link_id: 1,
            io_timeout: self.options.io_timeout.as_millis() as u32,
            lock_timeout: self.options.lock_timeout.as_millis() as u32,
            flags: 0,
            data
        };
        let resp: DeviceWriteResponse = rpc::call(&mut self.client, &request, PROG, VERS, CALL_DEVICE_WRITE).await?;
        if resp.error != 0 {
            Err(Error::VxiRemoteError(resp.error))
        } else {
            Ok(resp.size as usize)
        }
    }

}
