use std::ops::{Index, IndexMut};

use crate::window::{Window, Windows, Screen};
use crate::xserver::{XConn, XWindowID};

use crate::layout::{self, *};

#[derive(Clone)]
pub struct Workspace {
    pub(crate) windows: Windows,
    pub(crate) master: Option<XWindowID>,
    pub(crate) layout: LayoutType,

    pub _activate: fn(&XConn, &mut Workspace, &Screen),
    pub _deactivate: fn(&XConn, &mut Workspace),
    pub _add_window: fn(&XConn, &mut Workspace, &Screen, XWindowID),
    pub _del_window: fn(&XConn, &mut Workspace, &Screen, XWindowID, usize) -> Window,
    pub _focus_window: fn(&XConn, &mut Workspace, XWindowID),
    pub _relayout: fn(&XConn, &mut Workspace, &Screen),
    // pub _cycle_focus: fn(&XConn, &mut Workspace),
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            windows: Windows::default(),
            master: None,
            layout: LayoutType::DTiled,

            _activate: layout::activate,
            _deactivate: layout::deactivate,
            _add_window: dtiled::add_window,
            _del_window: dtiled::del_window,
            _focus_window: dtiled::window_focus,
            _relayout: dtiled::relayout,
            // _cycle_focus
        }
    }
}

#[allow(dead_code)]
impl Workspace {
    pub fn with_layout(layout: LayoutType) -> Self {
        match layout {
            LayoutType::Floating => Self {
                windows: Windows::default(),
                master: None,
                layout: layout,
    
                _activate: floating::activate,
                _deactivate: floating::deactivate,
                _add_window: floating::add_window,
                _del_window: floating::del_window,
                _focus_window: floating::window_focus,
                _relayout: floating::relayout,
            },
            LayoutType::DTiled => Self {
                windows: Windows::default(),
                master: None,
                layout: layout,

                _activate: dtiled::activate,
                _deactivate: dtiled::deactivate,
                _add_window: dtiled::add_window,
                _del_window: dtiled::del_window,
                _focus_window: dtiled::window_focus,
                _relayout: dtiled::relayout,
            },
            unhandled => {
                error!("Layout type {:?} not supported", unhandled);
                Self::default()
            }
        }
    }

    pub fn set_layout(&mut self, layout: LayoutType) {
        match layout {
            LayoutType::Floating => {
                self.layout = layout;
                self._activate = layout::activate;
                self._deactivate = layout::deactivate;
                self._add_window = floating::add_window;
                self._del_window = floating::del_window;
                self._focus_window = floating::window_focus;
                self._relayout = floating::relayout;
            }
            unhandled => {
                error!("Layout type {:?} not supported", unhandled)
            }
        }
    }

    pub fn push_window(&mut self, window: Window) {
        if let LayoutType::Floating = self.layout {
            self.windows.push(window);
        } else if let None = self.master {
            if !self.windows.is_empty() {
                warn!("Windows is not empty but workspace has a master")
            }
            self.set_master(window.id());
            self.windows.push(window);
        } else {
            self.windows.insert(1, window);
        }

    }

    pub fn set_master(&mut self, master_id: XWindowID) {
        self.master = Some(master_id);
    }

    pub fn unset_master(&mut self) {
        self.master = None;
    }

    #[inline]
    pub fn is_master(&self, win_id: XWindowID) -> bool {
        if let Some(win) = self.master {
            return win == win_id
        }
        false
    }

    #[inline(always)]
    pub fn master(&self) -> Option<XWindowID> {
        self.master
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.windows.len() == 0
    }

    pub fn is_tiling(&self) -> bool {
        if let LayoutType::Floating = self.layout {
            return false
        }
        true
    }

    pub fn is_floating(&self) -> bool {
        if let LayoutType::Floating = self.layout {
            return true
        }
        false
    }

    pub fn activate(&mut self, conn: &XConn, screen: &Screen) {
        (self._activate)(conn, self, screen);
    }

    pub fn deactivate(&mut self, conn: &XConn) {
        (self._deactivate)(conn, self);
    }

    pub fn add_window(&mut self, conn: &XConn, screen: &Screen, id: XWindowID) {
        (self._add_window)(conn, self, screen, id);
        debug!("Current master is {:?}", self.master);
        dbg!(&self.windows);
    }

    pub fn del_window(&mut self, 
        conn: &XConn, 
        screen: &Screen, 
        id: XWindowID, 
        idx: usize
    ) -> Window {
        let window = (self._del_window)(conn, self, screen, id, idx);
        debug!("Current master is {:?}", self.master);
        dbg!(&self.windows);
        window
    }

    pub fn take_focused_window(&mut self,
        conn: &XConn,
        screen: &Screen,
    ) -> Option<Window> {
        if let Some(window) = self.windows.focused() {
            let idx = self.windows.contains(window.id()).unwrap();
            let window = window.to_owned();
            self.del_window(conn, screen, window.id(), idx);

            Some(window)
        } else {
            None
        }
    }

    pub fn focus_window(&mut self, conn: &XConn, _screen: &Screen, id: XWindowID) {
        debug!("Focusing window in workspace {}", id);

        (self._focus_window)(conn, self, id);
    }

    pub fn relayout(&mut self, conn: &XConn, scr: &Screen) {
        (self._relayout)(conn, self, scr);
    }

    pub fn contains(&self, window: XWindowID) -> Option<usize> {
        self.windows.contains(window)
    }
}

impl Index<usize> for Workspace {
    type Output = Window;

    fn index(&self, idx: usize) -> &Window {
        &self.windows[idx]
    }
}

impl IndexMut<usize> for Workspace {
    fn index_mut(&mut self, idx: usize) -> &mut Window {
        &mut self.windows[idx]
    }
}