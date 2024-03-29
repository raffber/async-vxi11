use xdr_rs_serialize::de::XDRIn;
use xdr_rs_serialize::ser::XDROut;

use crate::rpc::{Deserialize, Serialize};
use crate::Error;

pub struct CreateLinkRequest {
    pub client_id: u32,
    pub lock: bool,
    pub lock_timeout_ms: u32,
    pub device: String,
}

impl Serialize for CreateLinkRequest {
    fn serialize(&self, out: &mut Vec<u8>) {
        self.client_id.write_xdr(out).unwrap();
        self.lock.write_xdr(out).unwrap();
        self.lock_timeout_ms.write_xdr(out).unwrap();
        self.device.write_xdr(out).unwrap();
    }
}

pub struct CreateLinkResponse {
    pub error: u32,
    pub link_id: u32,
    pub port: u32,
    pub max_recv_size: u32,
}

impl Deserialize for CreateLinkResponse {
    fn deserialize_partial(data: &[u8]) -> crate::Result<(Self, usize)> {
        let (error, _) = u32::read_xdr(&data[0..]).map_err(Error::XdrError)?;
        let (link_id, _) = u32::read_xdr(&data[4_usize..]).map_err(Error::XdrError)?;
        let (port, _) = u32::read_xdr(&data[8_usize..]).map_err(Error::XdrError)?;
        let (max_recv_size, _) = u32::read_xdr(&data[12_usize..]).map_err(Error::XdrError)?;

        let ret = CreateLinkResponse {
            error,
            link_id,
            port,
            max_recv_size,
        };
        Ok((ret, 16_usize))
    }
}

pub struct DeviceWriteRequest {
    pub link_id: u32,
    pub io_timeout: u32,
    pub lock_timeout: u32,
    pub flags: u32,
    pub data: Vec<u8>,
}

impl Serialize for DeviceWriteRequest {
    fn serialize(&self, out: &mut Vec<u8>) {
        self.link_id.write_xdr(out).unwrap();
        self.io_timeout.write_xdr(out).unwrap();
        self.lock_timeout.write_xdr(out).unwrap();
        self.flags.write_xdr(out).unwrap();
        self.data.write_xdr(out).unwrap();
    }
}

pub struct DeviceWriteResponse {
    pub error: u32,
    pub size: u32,
}

impl Deserialize for DeviceWriteResponse {
    fn deserialize_partial(data: &[u8]) -> crate::Result<(Self, usize)> {
        let (error, _) = u32::read_xdr(data).map_err(Error::XdrError)?;
        let (size, _) = u32::read_xdr(&data[4_usize..]).map_err(Error::XdrError)?;
        let ret = DeviceWriteResponse { error, size };
        Ok((ret, 8_usize))
    }
}

pub struct DeviceReadRequest {
    pub link_id: u32,
    pub request_size: u32,
    pub io_timeout: u32,
    pub lock_timeout: u32,
    pub flags: u32,
    pub term_char: u32,
}

impl Serialize for DeviceReadRequest {
    fn serialize(&self, out: &mut Vec<u8>) {
        self.link_id.write_xdr(out).unwrap();
        self.request_size.write_xdr(out).unwrap();
        self.io_timeout.write_xdr(out).unwrap();
        self.lock_timeout.write_xdr(out).unwrap();
        self.flags.write_xdr(out).unwrap();
        self.term_char.write_xdr(out).unwrap();
    }
}

pub struct DeviceReadResponse {
    pub error: u32,
    pub reason: u32,
    pub data: Vec<u8>,
}

impl Deserialize for DeviceReadResponse {
    fn deserialize_partial(data: &[u8]) -> crate::Result<(Self, usize)> {
        let (error, _) = u32::read_xdr(data).map_err(Error::XdrError)?;
        let (reason, _) = u32::read_xdr(&data[4..]).map_err(Error::XdrError)?;
        let (data, len) = Vec::<u8>::read_xdr(&data[8..]).map_err(Error::XdrError)?;
        let ret = DeviceReadResponse {
            error,
            reason,
            data,
        };
        Ok((ret, (len + 8) as usize))
    }
}
