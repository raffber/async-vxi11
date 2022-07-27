//! This crate provides an asynchronous *client* implementation for the the VXI-11 protcol.
//!
//! The transport layer is represented in the [`rpc::Client`] trait and implemented both for
//! `tokio` (with [`tokio::TcpClient`]) and `async-std` (with [`async_std::TcpClient`]).
//! The `tokio` (default) and the `async-std` features enable the conditional compilation of above modules.
//! It is also possible to deactive both features and to provide a custom [`rpc::Client`] implementation.
//!
//! Main rpc client is implemented with the [`CoreClient<T: Client>`][core::client::CoreClient]
//!
//! VXI-11 is a somewhat old and exotic protocol. It's a stack of a few technologies (specs linked):
//!  
//!  - [XDR](https://tools.ietf.org/html/rfc4506) - a very simple serialization format
//!  - [ONC-RPC](https://tools.ietf.org/html/rfc5531#section-9) - Also known as [SUN RPC](https://en.wikipedia.org/wiki/Sun_RPC). It uses XDR.
//!  - The [port mapper](https://tools.ietf.org/html/rfc1833) protocol is a protcol on top of ONC-RPC used to establish
//!     a connection to a server. A client first ask the sever over this protocol to which
//!     port it should connect. The port of the portmapper protocol is standardized to 111.
//!  - [VXI-11](https://www.vxibus.org/specifications.html) uses the port mapper protocol to connect a client to a server and adds additional RPC calls.
//!     However, most communication still behaves as a byte stream using a write and a read RPC.
//!
use std::io;

use thiserror::Error;

pub use crate::core::client::{CoreClient, VxiOptions};
pub use rpc::{Client, Deserialize, Serialize};

pub mod core;
pub mod portmapper;
pub mod rpc;

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "async-std")]
pub mod async_std;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error occurred: {0}")]
    Io(io::Error),
    #[error("Cannot unpack message")]
    CannotUnpack,
    #[error("RPC Error: {0}")]
    Rpc(onc_rpc::Error),
    #[error("Unexpected xid")]
    UnexpectedXid { expected: u32, actual: u32 },
    #[error("Wrong message type")]
    WrongMessageType,
    #[error("RPC denied")]
    RpcDenied,
    #[error("XDR error")]
    XdrError(xdr_rs_serialize::error::Error),
    #[error("Invalid Port Number")]
    InvalidPortNumber,
    #[error("VXI remote error")]
    VxiRemoteError(u32),
    #[error("Invalid RPC args")]
    RpcInvalidArgs,
}

pub type Result<T> = std::result::Result<T, Error>;
