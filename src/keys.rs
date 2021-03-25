use x11::keysym;
use xcb::xproto;

use crate::WM;

pub type Keybind = (xcb::ModMask, xcb::Keysym, fn(&mut WM));

pub const MODKEY: xcb::ModMask = xproto::MOD_MASK_4;

pub const KEYBINDS: &[Keybind] = &[
    (MODKEY, keysym::XK_1, |wm| {wm.goto_workspace(0);}),
    (MODKEY, keysym::XK_2, |wm| {wm.goto_workspace(1);}),
];

pub fn find_keybind(modm: xcb::ModMask, key: xcb::Keysym) -> Option<Keybind> {
    
    for kb in KEYBINDS {
        if kb.0 == modm && kb.1 == key {
            return Some(*kb)
        }
    }
    
    None
}

// fn run_cb(args: &[&str]) {

// }