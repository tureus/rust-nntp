extern crate bufstream;
extern crate native_tls;
#[macro_use] extern crate prettytable;

/// Classic code for reference
mod nntp;
pub use self::nntp::*;

/// Over-engineered magic code 8-)
pub mod capabilities;
pub mod client;
pub mod response;
pub mod stream;