use x11::keysym;
use xcb::xproto;

use crate::types::{Direction::*, Keybind};
use crate::utils;

pub const MODKEY: xcb::ModMask = xproto::MOD_MASK_4;
pub const SHIFT: xcb::ModMask = xproto::MOD_MASK_SHIFT;

pub const KEYBINDS: &[Keybind] = &[
    (MODKEY, keysym::XK_1, |wm| {wm.goto_workspace(0);}),
    (MODKEY, keysym::XK_2, |wm| {wm.goto_workspace(1);}),

    (MODKEY|SHIFT, keysym::XK_1, |wm| {wm.send_window_to(0)}),
    (MODKEY|SHIFT, keysym::XK_2, |wm| {wm.send_window_to(1)}),

    (MODKEY, keysym::XK_t, |wm| {wm.desktop.current_mut().toggle_focused_state(&wm.conn, &wm.screen)}),

    (MODKEY, keysym::XK_Return, |_| {utils::run_external(&["alacritty"])}),
    (MODKEY, keysym::XK_r, |_| {utils::run_external(&["dmenu_run", "-b"])}),

    (MODKEY, keysym::XK_Left, |wm| {wm.desktop.current_mut().cycle_focus(&wm.conn, Backward)}),
    (MODKEY, keysym::XK_Right, |wm| {wm.desktop.current_mut().cycle_focus(&wm.conn, Forward)}),

    (MODKEY, keysym::XK_w, utils::close_window),

    (MODKEY|SHIFT, keysym::XK_q, |wm| {wm.quit()}),
];