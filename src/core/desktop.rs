//! This module exports `Desktop` and `Screen`.
//! 
//! `Desktop` is the main type handled directly by `WindowManager`.
//! It encapsulates multiple workspaces, and handles sending
//! windows between workspaces.
//! 
//! `Screen` represents a physical monitor that X is connected to.
//! It encapsulates monitor resolution and is used by the tiling
//! algorithms to resize windows.

use crate::workspace::Workspace;
use crate::x::core::{XWindowID, XConn};
use crate::layout::LayoutType;
use crate::types::{Ring, Direction};
use crate::x::XWindow;

const MAX_WKSPACES: usize = 10;

/// The main `Desktop` used by `WindowManager`.
#[derive(Clone, Default)]
pub struct Desktop {
    workspaces: Ring<Workspace>,
    current: usize,
}

/// Represents a physical monitor.
#[derive(Clone, Copy)]
pub struct Screen {
    pub xwindow: XWindow,
    pub idx: i32,
}

impl Screen {
    pub fn new(screen_idx: i32, root_id: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(root_id),
            idx: screen_idx,
        }
    }
}

impl Desktop {
    /// Creates a new desktop with a specified layout.
    pub fn new(layout: LayoutType) -> Self {
        Self {
            workspaces: {
                let mut workspaces = Ring::with_capacity(MAX_WKSPACES);

                for _ in 0..MAX_WKSPACES {
                    workspaces.push(Workspace::with_layout(layout));
                }

                workspaces.set_focused(0);
                workspaces
            },
            current: 0,
        }
    }

    /// Returns a view into the current workspace.
    pub fn current(&self) -> &Workspace {
        &self.workspaces[self.current]
    }

    /// Returns a mutable view into the current workspace
    pub fn current_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current]
    }

    /// Get the workspace the window is on and its index inside it.
    pub fn retrieve(&mut self, window: XWindowID) -> Option<(&Workspace, usize)> {
        for ws in self.workspaces.iter() {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx))
            }
        }

        None
    }

    /// `Desktop::retrieve`, mutable edition.
    pub fn retrieve_mut(&mut self, window: XWindowID) -> Option<(&mut Workspace, usize)> {
        for ws in self.workspaces.iter_mut() {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx))
            }
        }

        None
    }

    pub fn cycle_workspace(&mut self, 
        conn: &XConn, 
        scr: &Screen, 
        direction: Direction
    ) {
        debug!("Cycling workspaces in direction {:?}", direction);
        self.workspaces.cycle_focus(direction);
        if let Some(i) = self.workspaces.focused_idx() {
            self.goto(conn, scr, i);
        } else {
            error!("Focused should be Some");
        }
    }

    /// Get a reference to a workspace by its index
    pub fn get(&self, idx: usize) -> Option<&Workspace> {
        if idx + 1 >= self.workspaces.len() {
            return None
        }

        Some(&self.workspaces[idx])
    }

    /// Get a mutable reference to a workspace by index.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Workspace> {
        if idx + 1 > self.workspaces.len() {
            return None
        }

        Some(&mut self.workspaces[idx])
    }

    /// Switch to a given workspace.
    pub fn goto(&mut self, conn: &XConn, scr: &Screen, idx: usize) {
        if self.current == idx {
            return
        }
        debug!("Goto desktop {}", idx);

        self.workspaces.get_mut(self.current).unwrap().deactivate(conn);
        
        self.current = idx;

        if let Some(ws) = self.get_mut(self.current) {
            ws.activate(conn, scr);
        } else {
            error!("No workspace found for index {}", idx);
        }
    }

    /// Send a window to a given workspace.
    pub fn send_window_to(&mut self, conn: &XConn, scr: &Screen, idx: usize) {
        debug!("Attempting to send window to workspace {}", idx);
        if let Some(window) = self.current_mut().take_focused_window(conn, scr) {
            debug!("Sending window {} to workspace {}", window.id(), idx);
            self.workspaces[idx].push_window(window);
        } else {
            debug!("No focused window for workspace {}", idx);
        }
        self.current_mut().relayout(conn, scr);
    }
}