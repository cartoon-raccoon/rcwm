#![allow(dead_code)]

use crate::layout::LayoutType;
use crate::xserver::XConn;

#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    Tiled,
    Floating,
}

impl From<LayoutType> for WindowState {

    #[inline]
    fn from(from: LayoutType) -> WindowState {
        if let LayoutType::Floating = from {
            return Self::Floating
        }

        Self::Tiled
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    Urgent,
    Focused,
    Unfocused,
}

/// The EWMH types a window can take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    Dropdown,
    Notification,
    Normal,
}

impl WindowType {
    pub fn from_atom(atom: xcb::Atom, conn: &XConn) -> Option<Self> {
        use WindowType::*;

        if atom == conn.atoms.WM_WINDOW_TYPE_DESKTOP {
            return Some(Desktop)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_DOCK {
            return Some(Dock)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_TOOLBAR {
            return Some(Toolbar)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_MENU {
            return Some(Menu)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_UTILITY {
            return Some(Utility)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_SPLASH {
            return Some(Splash)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_DIALOG {
            return Some(Dialog)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_DROPDOWN_MENU {
            return Some(Dropdown)
        } else if atom == conn.atoms.WM_WINDOW_TYPE_NOTIFICATION {
            return Some(Notification)
        }else if atom == conn.atoms.WM_WINDOW_TYPE_NORMAL {
            return Some(Normal)
        } else {
            None
        }
    }
}