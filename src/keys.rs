use std::thread;
use std::process::Command;

use x11::keysym;
use xcb::xproto;

use crate::WM;

pub type Keybind = (xcb::ModMask, xcb::Keysym, fn(&mut WM));

pub const MODKEY: xcb::ModMask = xproto::MOD_MASK_4;
pub const SHIFT: xcb::ModMask = xproto::MOD_MASK_SHIFT;

pub const KEYBINDS: &[Keybind] = &[
    (MODKEY, keysym::XK_1, |wm| {wm.goto_workspace(0);}),
    (MODKEY, keysym::XK_2, |wm| {wm.goto_workspace(1);}),

    (MODKEY|SHIFT, keysym::XK_1, |wm| {wm.send_window_to(0)}),
    (MODKEY|SHIFT, keysym::XK_2, |wm| {wm.send_window_to(1)}),

    (MODKEY, keysym::XK_t, |wm| {wm.desktop.current_mut().toggle_focused_state(&wm.conn, &wm.screen)}),

    (MODKEY, keysym::XK_Return, |_| {run_external(&["alacritty"])}),
    (MODKEY, keysym::XK_r, |_| {run_external(&["dmenu_run", "-b"])}),

    (MODKEY, keysym::XK_w, close_window),

    (MODKEY|SHIFT, keysym::XK_q, |wm| {wm.quit()}),
];

pub fn find_keybind(modm: xcb::ModMask, key: xcb::Keysym) -> Option<Keybind> {
    
    for kb in KEYBINDS {
        if kb.0 == modm && kb.1 == key {
            return Some(*kb)
        }
    }
    
    None
}

pub fn close_window(wm: &mut WM) {
    if let Some(window) = wm.desktop.current_mut().windows.focused() {
        wm.conn.destroy_window(&window);
    }
}

fn run_external(args: &'static [&str]) {
    thread::spawn(move || {
        debug!("Executing command {}", args[0]);
        let mut cmd = Command::new(args[0]);

        cmd.args(&args[1..]);

        match cmd.status() {
            Ok(status) => {
                debug!("Command exited with status {}", status)
            }
            Err(e) => {
                debug!("Error while executing command: {}", e)
            }
        }
    });
}