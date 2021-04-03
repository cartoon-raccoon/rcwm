use std::ops::{Index, IndexMut};

use crate::window::{Client, Windows};
use crate::desktop::Screen;
use crate::x::core::{XConn, XWindowID};
use crate::values;

use crate::layout::{self, *};

#[derive(Clone)]
pub struct Workspace {
    pub(crate) windows: Windows,
    pub(crate) master: Option<XWindowID>,
    pub(crate) layout: LayoutType,

    pub _activate: fn(&XConn, &mut Workspace, &Screen),
    pub _deactivate: fn(&XConn, &mut Workspace),
    pub _add_window: fn(&XConn, &mut Workspace, &Screen, XWindowID),
    pub _del_window: fn(&XConn, &mut Workspace, &Screen, XWindowID, usize) -> Client,
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
            LayoutType::DTiled => {
                self.layout = layout;
                self._activate = dtiled::activate;
                self._deactivate = dtiled::deactivate;
                self._add_window = dtiled::add_window;
                self._del_window = dtiled::del_window;
                self._focus_window = dtiled::window_focus;
                self._relayout = dtiled::relayout;
            }
            unhandled => {
                error!("Layout type {:?} not supported", unhandled)
            }
        }
    }

    pub fn push_window(&mut self, window: Client) {
        function_ends!("[start] workspace::push_window");
        if let LayoutType::Floating = self.layout {
            self.windows.push(window);
        } else if let None = self.master {
            if self.tiled_count() > 0 {
                warn!("Windows not empty but workspace has no master")
            }
            if window.is_tiled() {
                let window_id = window.id();
                self.windows.push(window);
                self.set_master(window_id);
            } else {
                self.windows.push(window);
            }
        } else {
            self.windows.append(window);
        }
        function_ends!("[end] workspace::push_window");
    }

    pub fn set_master(&mut self, master_id: XWindowID) {
        if !self.windows.has(master_id) {
            error!("set_master: No such window {}", master_id);
            return
        }
        self.master = Some(master_id);
        let idx = self.windows.contains(master_id).unwrap();
        self.windows.move_front(idx);
    }

    pub fn unset_master(&mut self) {
        if self.tiled_count() > 0 {
            error!("unset_master: Workspace still has tiled windows");
        }
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

    pub fn tiled_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_tiled()).count()
    }

    pub fn floating_count(&self) -> usize {
        self.windows.iter().filter(|win| win.is_floating()).count()
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
    ) -> Client {
        let window = (self._del_window)(conn, self, screen, id, idx);
        debug!("Current master is {:?}", self.master);
        dbg!(&self.windows);
        window
    }

    pub fn toggle_focused_state(&mut self, conn: &XConn, screen: &Screen) {
        debug!("Toggling state of focused window {:#?}", self.windows.focused());
        let master = self.master;
        // If we have a focused window
        if let Some(win) = self.windows.focused_mut() {
            // set a stack variable to avoid overlapping lifetimes
            let win_id = win.id();
            if win.is_floating() { //toggling to tiled
                debug!("Toggling window to tiled");
                win.toggle_state();
                // if we have no master
                if master.is_none() {
                    debug!("No master, setting master");
                    self.set_master(win_id);
                }
                // keep floating windows on top
            } else { //toggling to floating
                debug!("Toggling window to floating");

                // toggle state and stack above
                win.toggle_state();
                win.configure(conn, &values::stack_above());

                if self.tiled_count() == 0 && self.master.is_some() {
                    // if master is the only window
                    debug!("All windows are floating, unsetting master");
                    self.unset_master();
                } else if self.tiled_count() > 0 && master.is_some() {
                    // if window to toggle is master
                    if master.unwrap() == win_id {
                        debug!("Window to toggle is master, setting new master");
                        // we can get idx 1 and safely unwrap because windows.len() >= 2
                        let new_master = self.windows.get(1).expect("No window of idx 1").id();
                        self.set_master(new_master);
                    }
                } else {
                    assert!(master.is_none());
                }
            }
            self.relayout(conn, screen);
        }
    }

    pub fn take_focused_window(&mut self,
        conn: &XConn,
        screen: &Screen,
    ) -> Option<Client> {
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
    type Output = Client;

    fn index(&self, idx: usize) -> &Client {
        &self.windows[idx]
    }
}

impl IndexMut<usize> for Workspace {
    fn index_mut(&mut self, idx: usize) -> &mut Client {
        &mut self.windows[idx]
    }
}