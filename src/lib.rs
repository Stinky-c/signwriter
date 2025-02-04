#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod client;
mod components;
mod models;
#[allow(clippy::all)] // Disable Clippy on auto-generated code
mod proto;
mod thread;
mod widgets;

pub use app::App;
