use std::net::IpAddr;
use std::time::Duration;

use crate::{Error, rpc};
use crate::core::calls::{CreateLinkRequest, CreateLinkResponse, DeviceReadRequest, DeviceReadResponse, DeviceWriteRequest, DeviceWriteResponse};
use crate::tcp_client::TcpClient;

const PROG: u32 = 0x0607af;
const VERS: u32 = 1;

const CALL_CREATE_LINK: u32 = 10;
const CALL_DESTROY_LINK: u32 = 23;
const CALL_DEVICE_WRITE: u32 = 11;
const CALL_DEVICE_READ: u32 = 12;

const IO_TIMEOUT_MS: u64 = 1000;

const OP_FLAG_END: u32 = 8;
const OP_FLAG_TERMCHAR_SET: u32 = 128;

const RX_CHR: u32 = 2;
const RX_END: u32 = 4;


#[derive(Clone)]
pub struct VxiOptions {
    termchr: Option<u8>,
    lock_timeout: Duration,
    io_timeout: Duration,
}

impl Default for VxiOptions {
    fn default() -> Self {
        Self {
            termchr: None,
            lock_timeout: Default::default(),
            io_timeout: Duration::from_millis(IO_TIMEOUT_MS),
        }
    }
}

pub struct CoreClient {
    client: TcpClient,
    abort_port: u16,
    pub options: VxiOptions,
    max_recv_size: u32,
    link_id: u32,
    client_id: u32,
}


impl CoreClient {
    pub async fn connect<T: Into<IpAddr>>(addr: T) -> crate::Result<Self> {
        let client = TcpClient::connect_with_mapper(addr, PROG, VERS).await?;

        let rnd1 = rand::random::<u16>() as u32;
        let rnd2 = rand::random::<u16>() as u32;

        let mut ret = Self {
            client,
            abort_port: 0,
            options: Default::default(),
            max_recv_size: 0,
            client_id: rnd1 + rnd2 + 1,
            link_id: 0
        };
        ret.create_link(false, Default::default()).await?;
        Ok(ret)
    }

    async fn create_link(&mut self, lock: bool, lock_timeout: Duration) -> crate::Result<()> {
        let req = CreateLinkRequest {
            client_id: self.client_id,
            lock,
            lock_timeout_ms: lock_timeout.as_millis() as u32,
            device: "instr".to_string(),
        };
        let resp: CreateLinkResponse = rpc::call(&mut self.client, &req, PROG, VERS, CALL_CREATE_LINK).await?;
        self.link_id = resp.link_id;
        self.max_recv_size = resp.max_recv_size.min(1024 * 1024);
        if resp.port < 65535 {
            self.abort_port = resp.port as u16;
        } else {
            return Err(Error::InvalidPortNumber);
        }
        if resp.error != 0 {
            Err(Error::VxiRemoteError(resp.error))
        } else {
            Ok(())
        }
    }

    pub async fn destroy_link(mut self) -> crate::Result<()> {
        let err: u32 = rpc::call(&mut self.client, &self.link_id, PROG, VERS, CALL_DESTROY_LINK).await?;
        if err != 0 {
            Err(Error::VxiRemoteError(err))
        } else {
            Ok(())
        }
    }

    pub async fn device_write(&mut self, data: Vec<u8>) -> crate::Result<()> {
        let mut data = data;
        let mut flags = 0_u32;
        while data.len() > 0 {
            // slice data up in multiple chunks
            let max_idx = data.len().min(self.max_recv_size as usize);
            let send: Vec<u8> = data.drain(0..max_idx).collect();
            if data.len() == 0 {
                flags |= OP_FLAG_END;
            }
            let request = DeviceWriteRequest {
                link_id: self.link_id,
                io_timeout: self.options.io_timeout.as_millis() as u32,
                lock_timeout: self.options.lock_timeout.as_millis() as u32,
                flags,
                data: send,
            };
            let resp: DeviceWriteResponse = rpc::call(&mut self.client, &request, PROG, VERS, CALL_DEVICE_WRITE).await?;
            if resp.error != 0 {
                return Err(Error::VxiRemoteError(resp.error));
            }
        }
        Ok(())
    }

    pub async fn device_read(&mut self) -> crate::Result<Vec<u8>> {
        let request_size = self.max_recv_size;
        let mut flags = 0_u32;
        let mut term_char = 0_u32;
        if let Some(term) = self.options.termchr {
            term_char = term as u32;
            flags |= OP_FLAG_TERMCHAR_SET;
        }

        let mut ret = Vec::new();
        loop {
            let request = DeviceReadRequest {
                link_id: self.link_id,
                request_size,
                io_timeout: self.options.io_timeout.as_millis() as u32,
                lock_timeout: self.options.lock_timeout.as_millis() as u32,
                flags,
                term_char,
            };
            let resp: DeviceReadResponse = rpc::call(&mut self.client, &request, PROG, VERS, CALL_DEVICE_READ).await?;
            if resp.error != 0 {
                return Err(Error::VxiRemoteError(resp.error));
            }
            ret.extend(resp.data);
            if resp.reason & (RX_END | RX_CHR) != 0 {
                break;
            }
        }
        Ok(ret)
    }
}
