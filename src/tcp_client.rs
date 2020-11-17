use std::io;
use std::io::Cursor;
use std::net::{IpAddr, SocketAddr};

use async_trait::async_trait;
use byteorder::{BigEndian, ByteOrder};
use bytes::{Bytes, BytesMut};
use onc_rpc::{AcceptedStatus, MessageType, ReplyBody, RpcMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::prelude::{AsyncRead, AsyncWrite};

use crate::Error;
use crate::portmapper::PortMapper;
use crate::rpc::Client;
use crate::rpc::Request;

async fn send_record<T: AsyncWrite + Unpin, D: AsRef<[u8]>>(sock: &mut T, data: D) -> io::Result<()> {
    let data = data.as_ref();
    sock.write_all(&data).await
}

async fn recv_record<T: AsyncRead + Unpin>(sock: &mut T) -> io::Result<Bytes> {
    let mut ret = BytesMut::new();
    loop {
        let mut header_data = [0_u8; 4];
        sock.read_exact(&mut header_data).await?;
        let header = BigEndian::read_u32(&header_data);
        let num = header & 0x7fffffff;
        let mut buf = vec![0_u8; num as usize];
        sock.read_exact(&mut buf).await?;

        ret.reserve((num + 4) as usize);
        ret.extend_from_slice(&header_data);
        ret.extend_from_slice(&buf);
        if header & 0x80000000 != 0 {
            break;
        }
    }
    Ok(ret.freeze())
}

pub struct TcpClient {
    stream: TcpStream,
    xid: u32,
}

impl TcpClient {
    pub async fn connect<T: Into<SocketAddr>>(addr: T) -> crate::Result<Self> {
        let stream = TcpStream::connect(addr.into()).await.map_err(Error::Io)?;
        Ok(Self {
            stream,
            xid: 0,
        })
    }

    pub async fn connect_with_mapper<T: Into<IpAddr>>(addr: T, prog: u32, vers: u32) -> crate::Result<Self> {
        let addr = addr.into();
        let mapper_addr = SocketAddr::new(addr, 111);
        let mapper_client = TcpClient::connect(mapper_addr).await?;
        let mut mapper = PortMapper::new(mapper_client);
        let port = mapper.get_port(prog, vers).await?;
        let addr = SocketAddr::new(addr, port);
        TcpClient::connect(addr).await
    }
}

#[async_trait]
impl Client for TcpClient {
    async fn call(&mut self, body: Request) -> crate::Result<Bytes> {
        self.xid += 1;

        // construct a message and serialize
        let msg = RpcMessage::new(self.xid, MessageType::Call(body));
        let buf = Vec::with_capacity(msg.serialised_len() as usize);
        let mut cursor = Cursor::new(buf);
        msg.serialise_into(&mut cursor).map_err(Error::Io)?;

        // send data out
        send_record(&mut self.stream, &cursor.into_inner()).await.map_err(Error::Io)?;

        // keep receiving data
        loop {
            let reply = recv_record(&mut self.stream).await.map_err(Error::Io)?;
            let msg = RpcMessage::from_bytes(&reply).map_err(Error::Rpc)?;
            if msg.xid() < self.xid {
                continue;
            } else if msg.xid() > self.xid {
                return Err(Error::UnexpectedXid {
                    expected: self.xid,
                    actual: msg.xid(),
                });
            }
            // msg.xid() == self.xid()
            return if let Some(body) = msg.reply_body() {
                match body {
                    ReplyBody::Accepted(x) => {
                        let status = x.status();
                        match status {
                            AcceptedStatus::Success(data) => Ok(Bytes::copy_from_slice(data)),
                            _ => Err(Error::RpcInvalidArgs)
                        }
                    }
                    ReplyBody::Denied(_) => {
                        Err(Error::RpcDenied)
                    }
                }
            } else {
                Err(Error::WrongMessageType)
            };
        }
    }
}
