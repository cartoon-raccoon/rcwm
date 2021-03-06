#[macro_use]
mod log;

pub mod core;
pub mod layout;
pub mod x;
pub mod config;

pub mod utils;
pub mod manager;

pub use crate::manager::WindowManager;

pub use crate::core::{
    window, workspace, desktop, types,
};