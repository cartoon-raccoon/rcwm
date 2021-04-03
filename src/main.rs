#![allow(unused_imports)]

#[macro_use]
mod log;

mod wm;
mod x;
mod core;
mod utils;
mod layout;
mod config;

pub use crate::core::{
    window, workspace, desktop, types,
};

#[macro_use]
extern crate log as logger;

use wm::WindowManager;

use xcb::base::Connection;
use xcb_util::ewmh;

use nix::sys::signal::{
    signal, Signal, SigHandler
};

fn main() -> anyhow::Result<()> {
    unsafe {
        signal(Signal::SIGINT, SigHandler::SigIgn)?;
        signal(Signal::SIGQUIT, SigHandler::SigIgn)?;
    }

    let (conn, screen_idx) = Connection::connect(None)
        .expect("Failed to connect to X server");
    
    let conn = ewmh::Connection::connect(conn)
        .map_err(|(err, _)| { err })
        .expect("Failed to connect via EWMH");

    let mut wm = WindowManager::register(&conn, screen_idx);

    wm.run();

    Ok(())
}
