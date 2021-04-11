//! Various data types and definitions for use within RaccoonWM.
use xcb_util::icccm::{self, WmState};
use xcb::xproto;

use std::ops::Deref;

use crate::layout::LayoutType;
use crate::x::{XConn, Atom};
use crate::WindowManager;

pub use crate::core::{Ring, Selector};

/// Encapsulates a Keybind: The modifier, the actual key, and the callback to run.
pub type Keybind = (xcb::ModMask, xcb::Keysym, fn(&mut WindowManager));

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ModKey {
    Meta,
    Alt,
}

/// The Shift keymask.
pub const SHIFT: xcb::ModMask = xproto::MOD_MASK_SHIFT;
/// The Ctrl keymask.
pub const CTRL: xcb::ModMask = xproto::MOD_MASK_CONTROL;

/// Sets the Modkey to be used.
pub const fn modkey(modkey: ModKey) -> xcb::ModMask {
    use ModKey::*;
    match modkey {
        Meta => xproto::MOD_MASK_4,
        Alt  => xproto::MOD_MASK_1,
    }
}

/// Represents a direction to move.
/// 
/// Used to cycle focus between windows, or cycle windows.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

/// A specific point on a window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// The Geometry of a given window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<(i32, i32, i32, i32)> for Geometry {
    fn from(from: (i32, i32, i32, i32)) -> Self {
        Self {
            x: from.0,
            y: from.1,
            width: from.2,
            height: from.3,
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Geometry {
            x: 0,
            y: 0,
            width: 160,
            height: 100,
        }
    }
}

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

/// Convenience wrapper around a Vec of NetWindowStates.
#[derive(Debug, Clone)]
pub struct NetWindowStates {
    states: Vec<Atom>,
}

impl NetWindowStates {
    pub fn new() -> Self {
        Self {
            states: Vec::new()
        }
    }

    pub fn contains(&self, prop: Atom) -> bool {
        self.states.contains(&prop)
    }

    pub fn add(&mut self, prop: Atom) {
        self.states.push(prop)
    }

    pub fn remove(&mut self, prop: Atom) -> Atom {
        for (idx, atom) in self.states.iter().enumerate() {
            if *atom == prop {
                return self.states.remove(idx)
            }
        }
        error!("Tried to remove atom not in states");
        0
    }
}

impl From<Vec<Atom>> for NetWindowStates {
    fn from(from: Vec<Atom>) -> Self {
        Self {
            states: from
        }
    }
}

impl Deref for NetWindowStates {
    type Target = [Atom];

    fn deref(&self) -> &Self::Target {
        self.states.as_slice()
    }
}

//? This thing's existence is in question.
/// EWMH-defined window states.
#[derive(Clone, Copy, Debug)]
pub enum NetWindowState {
    Modal,
    Sticky,
    MaxVert,
    MaxHorz,
    Shaded,
    SkipTaskbar,
    SkipPager,
    Hidden,
    Fullscreen,
    Above,
    Below,
    Urgent,
}

impl NetWindowState {
    pub fn from_atom(atom: Atom, conn: &XConn) -> Option<Self> {
        let raw = conn.get_raw();
        if atom == raw.WM_STATE_MODAL() {
            return Some(Self::Modal)
        } else if atom == raw.WM_STATE_STICKY() {
            return Some(Self::Sticky)
        } else if atom == raw.WM_STATE_MAXIMIZED_VERT() {
            return Some(Self::MaxVert)
        } else if atom == raw.WM_STATE_MAXIMIZED_HORZ() {
            return Some(Self::MaxHorz)
        } else if atom == raw.WM_STATE_SHADED() {
            return Some(Self::Shaded)
        } else if atom == raw.WM_STATE_SKIP_TASKBAR() {
            return Some(Self::SkipTaskbar)
        } else if atom == raw.WM_STATE_SKIP_PAGER() {
            return Some(Self::SkipPager)
        } else if atom == raw.WM_STATE_HIDDEN() {
            return Some(Self::Hidden)
        } else {
            return None
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Used internally to track whether a window is floating or tiled.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WinLayoutState {
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

/// The style for the window border.
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
    /// Gets the atom type from an atom and an XConn.
    pub fn from_atom(atom: Atom, conn: &XConn) -> Option<Self> {
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

//todo
/// ICCCM-defined window size hints.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SizeHints {
    pub position: Option<(i32, i32)>,
    pub size: Option<(i32, i32)>,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub resize: Option<(i32, i32)>,
    pub min_aspect: Option<(i32, i32)>,
    pub max_aspect: Option<(i32, i32)>,
    pub base: Option<(i32, i32)>,
    pub gravity: Option<u32>
}

/// ICCCM-defined window hints.
#[derive(Debug, Clone, Copy)]
pub struct WmHints {
    pub state: WindowState,
    pub urgent: bool,
    //todo: add pixmaps
}

/// ICCCM-defined window properties.
//todo: make all fields private, accessible with methods.
pub struct XWinProperties {
    pub(crate) wm_name: String,
    pub(crate) wm_icon_name: String,
    pub(crate) wm_size_hints: Option<icccm::SizeHints>,
    pub(crate) wm_hints: Option<icccm::WmHints>,
    pub(crate) wm_class: (String, String), //Instance, Class
    pub(crate) wm_protocols: Option<Vec<Atom>>,
    pub(crate) wm_state: WindowState,
}

impl XWinProperties {
    pub fn wm_name(&self) -> &str {
        &self.wm_name
    }

    pub fn wm_icon_name(&self) -> &str {
        &self.wm_icon_name
    }

    #[inline]
    pub fn wm_size_hints(&self) -> Option<SizeHints> {
        if let Some(hints) = &self.wm_size_hints {
            Some(SizeHints {
                position: hints.position(),
                size: hints.size(),
                min_size: hints.min_size(),
                max_size: hints.max_size(),
                resize: hints.resize(),
                min_aspect: hints.aspect().map(|(i,_)| i),
                max_aspect: hints.aspect().map(|(_,i)| i),
                base: hints.base(),
                gravity: hints.gravity(),
            })
        } else {
            None
        }
    }

    pub fn wm_hints(&self) -> Option<WmHints> {
        use WindowState::*;
        if let Some(hints) = &self.wm_hints {
            Some(WmHints {
                state: if hints.is_normal() {
                    Normal
                } else if hints.is_iconic() {
                    Iconic
                } else if hints.is_withdrawn() {
                    Withdrawn
                } else {
                    Normal
                },
                urgent: if let Some(u) = hints.is_urgent() {u} else {false}
            })
        } else {
            None
        }
    }

    pub fn wm_class(&self) -> (&str, &str) {
        (&self.wm_class.0, &self.wm_class.1)
    }

    pub fn window_type(&self) -> Option<&[Atom]> {
        if let Some(prtcls) = &self.wm_protocols {
            return Some(&prtcls)
        } else {
            None
        }
    }

    pub fn wm_state(&self) -> WindowState {
        self.wm_state
    }
}