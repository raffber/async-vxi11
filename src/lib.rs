use std::io;
use thiserror::Error;

pub mod tcp_client;
mod rpc;
mod portmapper;
mod core;

pub use core::client::{CoreClient, VxiOptions};

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
        actual: u32
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
    #[error("Write call did not complete to write all data")]
    WriteFailedToWriteBuffer,
}


pub type Result<T> = std::result::Result<T, Error>;
