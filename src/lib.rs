#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod client;
#[allow(clippy::all)] // Disable Clippy on auto-generated code
mod proto;
mod thread;

pub use app::App;
