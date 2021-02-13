use crate::window::{Window, Windows, Screen};
use crate::xserver::{XConn, XWindowID};

use crate::layout::*;

pub struct Workspace {
    pub(crate) windows: Windows,

    pub _activate: fn(&XConn, &mut Workspace, &Screen),
    pub _deactivate: fn(&XConn, &mut Workspace),
    pub _add_window: fn(&XConn, &mut Workspace, &Screen, XWindowID),
    pub _del_window: fn(&XConn, &mut Workspace, &Screen, XWindowID, usize) -> Window,
    pub _focus_window: fn(&XConn, &mut Workspace, XWindowID),
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            windows: Windows::default(),

            _activate: floating::activate,
            _deactivate: floating::deactivate,
            _add_window: floating::add_window,
            _del_window: floating::del_window,
            _focus_window: floating::window_focus,
        }
    }
}

impl Workspace {
    pub fn set_layout(&mut self, layout: LayoutType) {
        match layout {
            LayoutType::Floating => {
                self._activate = floating::activate;
                self._deactivate = floating::deactivate;
                self._add_window = floating::add_window;
                self._del_window = floating::del_window;
                self._focus_window = floating::window_focus;
            }
            unhandled => {
                error!("Layout type {:?} not supported", unhandled)
            }
        }
    }
    pub fn activate(&mut self, conn: &XConn, screen: &Screen) {
        (self._activate)(conn, self, screen);
    }

    pub fn deactivate(&mut self, conn: &XConn) {
        (self._deactivate)(conn, self);
    }

    pub fn add_window(&mut self, conn: &XConn, screen: &Screen, id: XWindowID) {
        (self._add_window)(conn, self, screen, id);
    }

    pub fn del_window(&mut self, 
        conn: &XConn, 
        screen: &Screen, 
        id: XWindowID, 
        idx: usize
    ) -> Window {
        (self._del_window)(conn, self, screen, id, idx)
    }
}