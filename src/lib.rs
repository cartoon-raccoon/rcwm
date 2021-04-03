#[macro_use]
pub mod log;

pub mod core;
pub mod layout;
pub mod x;
pub mod config;

pub mod utils;
pub mod wm;

pub use crate::wm::WindowManager;

pub use crate::core::{
    window, workspace, desktop, types,
};