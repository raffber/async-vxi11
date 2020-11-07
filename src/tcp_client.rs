use crate::client::Client;
use async_trait::async_trait;
use tokio::prelude::{AsyncWrite, AsyncRead};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use byteorder::{ByteOrder, BigEndian};
use crate::Error;
use bytes::{Bytes, BytesMut};
use tokio::net::{TcpSocket, TcpStream};
use std::net::SocketAddr;
use crate::client::Request;
use std::io::Cursor;
use std::io;
use onc_rpc::{RpcMessage, ReplyBody};

const DEVICE_CORE_PROG: u32 = 0x0607af;

struct RpcInfo {
    prog: u32,
    vers: u32,
}

async fn send_record<T: AsyncWrite + Unpin, D: AsRef<[u8]>>(sock: &mut T, data: D) -> io::Result<()> {
    let data = data.as_ref();
    let len = data.len();
    let mut buf= [0; 4];
    let starter = len as u32 | 0x80000000_u32;
    BigEndian::write_u32(&mut buf, starter);
    sock.write_all(&buf).await?;
    sock.write_all(data.as_ref()).await
}

async fn recv_record<T: AsyncRead + Unpin>(sock: &mut T) -> io::Result<Bytes> {
    let mut ret = BytesMut::new();
    loop {
        let header = sock.read_u32().await?;
        let num =  header & 0x7fffffff;
        ret.reserve(num as usize);
        let mut buf = vec![0_u8; num as usize];
        sock.read_exact(&mut buf).await?;
        ret.extend_from_slice(&buf);
        if header & 0x80000000 != 0 {
            break;
        }
    }
    Ok(ret.freeze())
}

struct TcpClient {
    stream: TcpStream,
    info: RpcInfo,
    xid: u32,
}

impl TcpClient {
    pub async fn connect<T: Into<SocketAddr>>(addr: T, info: RpcInfo) -> crate::Result<Self> {
        let socket = TcpSocket::new_v4().map_err(Error::Io)?;
        let stream = socket.connect(addr.into()).await.map_err(Error::Io)?;
        Ok(Self {
            stream,
            info,
            xid: 0
        })
    }
}

#[async_trait]
impl Client for TcpClient {
    async fn call(&mut self, body: Request) -> crate::Result<Bytes> {
        let mut cursor = Cursor::new(Vec::with_capacity(body.serialised_len() as usize));
        body.serialise_into(&mut cursor).map_err(Error::Io)?;
        send_record(&mut self.stream, &cursor.into_inner()).await.map_err(Error::Io)?;
        loop {
            let reply = recv_record(&mut self.stream).await.map_err(Error::Io)?;
            let msg = RpcMessage::from_bytes(&reply).map_err(Error::Rpc)?;
            if msg.xid() < self.xid {
                continue
            } else {
                return Err(Error::UnexpectedXid {
                    expected: self.xid,
                    actual: msg.xid()
                })
            }
            return if let Some(body) = msg.reply_body() {
                match body {
                    ReplyBody::Accepted(x) => {
                        Ok(reply.slice(x.serialised_len() as usize..))
                    },
                    ReplyBody::Denied(_) => {
                        Err(Error::RpcDenied)
                    },
                }
            } else {
                Err(Error::WrongMessageType)
            }
        }
    }
}
