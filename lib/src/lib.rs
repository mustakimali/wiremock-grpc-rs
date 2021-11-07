#![doc = include_str!("../README.md")]
mod builder;
mod codegen;
mod grpc_server;
mod invocations;
mod tonic_ext;

pub use builder::{MockBuilder, Mountable, Then};
pub use grpc_server::GrpcServer;

pub extern crate http_body;
pub extern crate tonic;
