#![allow(dead_code)]

use xcb_util::icccm::{self, WmState};

use crate::layout::LayoutType;
use crate::xserver::XConn;

/// The ICCCM-defined window states.
#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    Normal,
    Withdrawn,
    Iconic,
}

impl From<WmState> for WindowState {
    fn from(from: WmState) -> Self {
        match from {
            icccm::WM_STATE_NORMAL => Self::Normal,
            icccm::WM_STATE_WITHDRAWN => Self::Withdrawn,
            icccm::WM_STATE_ICONIC => Self::Iconic,
            _ => unreachable!("Unknown state")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum WinLayoutState {
    Tiled,
    Floating,
}

impl From<LayoutType> for WinLayoutState {

    #[inline]
    fn from(from: LayoutType) -> WinLayoutState {
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

pub struct SizeHints {
    pub position: Option<(i32, i32)>,
    pub size: Option<(i32, i32)>,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub resize: Option<(i32, i32)>,
    pub aspect: Option<(i32, i32)>,
}

pub struct XWinProperties {
    pub wm_name: String,
    pub wm_icon_name: String,
    pub wm_size_hints: icccm::SizeHints,
    pub wm_hints: icccm::WmHints,
    pub wm_class: (String, String), //Instance, Class
    pub wm_protocols: Vec<xcb::Atom>,
    pub wm_state: WindowState,
}