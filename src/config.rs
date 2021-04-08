use x11::keysym;
use xcb::xproto;

use crate::types::{Direction::*, Keybind};
use crate::utils;

pub const MODKEY: xcb::ModMask = xproto::MOD_MASK_4;
pub const SHIFT: xcb::ModMask = xproto::MOD_MASK_SHIFT;
pub const CTRL: xcb::ModMask = xproto::MOD_MASK_CONTROL;
pub const BORDER_WIDTH: u32 = 2;

pub const BORDER_FOCUSED: u32 = 0xdddddd;
pub const BORDER_UNFOCUSED: u32 = 0x555555;
pub const BORDER_URGENT: u32 = 0xff0000;

pub const KEYBINDS: &[Keybind] = &[
    (MODKEY, keysym::XK_1, |wm| {wm.goto_workspace(0);}),
    (MODKEY, keysym::XK_2, |wm| {wm.goto_workspace(1);}),
    (MODKEY, keysym::XK_3, |wm| {wm.goto_workspace(2);}),
    (MODKEY, keysym::XK_4, |wm| {wm.goto_workspace(3);}),
    (MODKEY, keysym::XK_5, |wm| {wm.goto_workspace(4);}),

    (MODKEY|SHIFT, keysym::XK_1, |wm| {wm.send_window_to(0)}),
    (MODKEY|SHIFT, keysym::XK_2, |wm| {wm.send_window_to(1)}),
    (MODKEY|SHIFT, keysym::XK_3, |wm| {wm.send_window_to(2)}),
    (MODKEY|SHIFT, keysym::XK_4, |wm| {wm.send_window_to(3)}),
    (MODKEY|SHIFT, keysym::XK_5, |wm| {wm.send_window_to(4)}),

    (MODKEY, keysym::XK_t, |wm| {wm.toggle_focused_state()}),

    (MODKEY, keysym::XK_Return, |_| {utils::run_external(&["alacritty"])}),
    (MODKEY, keysym::XK_r, |_| {utils::run_external(&["dmenu_run", "-b"])}),

    (MODKEY, keysym::XK_Left, |wm| {wm.cycle_focus(Backward)}),
    (MODKEY, keysym::XK_Right, |wm| {wm.cycle_focus(Forward)}),

    (MODKEY|SHIFT, keysym::XK_Left, |wm| {wm.cycle_workspace(Backward)}),
    (MODKEY|SHIFT, keysym::XK_Right, |wm| {wm.cycle_workspace(Forward)}),

    (MODKEY|CTRL, keysym::XK_Left, |wm| {wm.cycle_master(Backward)}),
    (MODKEY|CTRL, keysym::XK_Right, |wm| {wm.cycle_master(Forward)}),

    (MODKEY, keysym::XK_w, utils::close_window),

    (MODKEY|SHIFT, keysym::XK_q, |wm| {wm.quit()}),
];