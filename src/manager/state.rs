use super::{WindowManager, MouseMode};

use crate::core::{Ring, Workspace};
use crate::x::{XCBConnection, XWindowID};


pub(crate) struct WMState<'a> {
    pub conn: &'a XCBConnection,
    pub workspaces: &'a Ring<Workspace>,
    pub root: i32,
    pub mousemode: MouseMode,
    pub selected: Option<XWindowID>,
}

impl WindowManager {
    pub(crate) fn state(&self) -> WMState {
        WMState {
            conn: &self.conn,
            workspaces: &self.desktop.workspaces,
            root: self.root,
            mousemode: self.mousemode,
            selected: self.selected,
        }
    }
}