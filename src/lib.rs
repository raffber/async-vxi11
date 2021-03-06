use std::io;

use thiserror::Error;

pub use crate::core::client::{CoreClient, VxiOptions};

pub mod tcp_client;
mod rpc;
mod portmapper;
mod core;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error occurred: {0}")]
    Io(io::Error),
    #[error("Cannot unpack message")]
    CannotUnpack,
    #[error("RPC Error: {0}")]
    Rpc(onc_rpc::Error),
    #[error("Unexpected xid")]
    UnexpectedXid {
        expected: u32,
        actual: u32,
    },
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
