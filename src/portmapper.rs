
use crate::client::Client;
use xdr_rs_serialize::ser::XDROut;
use xdr_rs_serialize::de::XDRIn;
use onc_rpc::CallBody;
use onc_rpc::auth::AuthFlavor;

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

impl Mapping {
    fn write_xdr(&self, out: &mut Vec<u8>) {
        // actually cannot fail, because write_xdr call vec::write
        // which does not fail
        u32::write_xdr(&self.prog, out).unwrap();
        u32::write_xdr(&self.vers, out).unwrap();
        u32::write_xdr(&self.prot, out).unwrap();
        u32::write_xdr(&self.port, out).unwrap();
    }
}

pub enum IPProtocol {
    TCP, UDP
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
        // formulate request
        let request = Mapping {
            prog,
            vers,
            prot: protocol.protid(),
            port: 0
        };
        let mut payload = Vec::new();
        request.write_xdr(&mut payload);

        // call
        let req = self.client.make_request(PROG, VERS, PROC_GETPORT, payload);
        let data = self.client.call(req).await?;

        // deserialize
        let (ret, _) = u32::read_xdr(&data).map_err(crate::Error::XdrError)?;
        if ret > 65535 {
            return Err(crate::Error::InvalidPortNumber);
        }
        Ok(ret as u16)
    }

}