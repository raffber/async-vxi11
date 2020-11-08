use onc_rpc::auth::AuthFlavor;
use onc_rpc::CallBody;
use xdr_rs_serialize::de::XDRIn;
use xdr_rs_serialize::ser::XDROut;

use crate::rpc::{Client, Serialize};
use crate::rpc;

const PROG: u32 = 100000;
const VERS: u32 = 2;

const PROC_GETPORT: u32 = 3;

const IPPROTO_TCP: u32 = 6;
const IPPROTO_UDP: u32 = 17;

struct Mapping {
    prog: u32,
    vers: u32,
    prot: u32,
    port: u32,
}


impl Serialize for Mapping {
    fn serialize(&self, out: &mut Vec<u8>) {
        // actually cannot fail, because write_xdr call vec::write
        // which does not fail
        u32::write_xdr(&self.prog, out).unwrap();
        u32::write_xdr(&self.vers, out).unwrap();
        u32::write_xdr(&self.prot, out).unwrap();
        u32::write_xdr(&self.port, out).unwrap();
    }
}


pub enum IPProtocol {
    TCP,
    UDP,
}

impl IPProtocol {
    fn protid(&self) -> u32 {
        match self {
            IPProtocol::TCP => IPPROTO_TCP,
            IPProtocol::UDP => IPPROTO_UDP,
        }
    }
}

pub struct PortMapper<C: Client> {
    client: C
}

impl<C: Client> PortMapper<C> {
    pub fn new(client: C) -> Self {
        Self {
            client
        }
    }

    pub async fn get_port(&mut self, prog: u32, vers: u32, protocol: IPProtocol) -> crate::Result<u16> {
        let request = Mapping {
            prog,
            vers,
            prot: protocol.protid(),
            port: 0,
        };

        let ret: u32 = rpc::call(&mut self.client, &request, PROG, VERS, PROC_GETPORT).await?;

        if ret > 65535 {
            return Err(crate::Error::InvalidPortNumber);
        }
        Ok(ret as u16)
    }
}