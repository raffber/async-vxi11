use crate::tcp_client::TcpClient;
use std::net::IpAddr;

const PROG: u32 = 0x0607af;
const VERS: u32 = 1;

struct CoreClient {
   client: TcpClient,
}

impl CoreClient {
   pub async fn connect<T: Into<IpAddr>>(addr: T) -> crate::Result<Self> {
      let client = TcpClient::connect_with_mapper(addr, PROG, VERS).await?;
      Ok(Self {
         client
      })
   }
}
