//! Selected local MCP boundary; operation semantics remain in the trusted runners.
mod framing;
mod handler;
mod output;
mod schema;
mod service;
mod transport;

pub use transport::run;
