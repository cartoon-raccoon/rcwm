use crate::workspace::Workspace;
use crate::xserver::XWindowID;
use crate::layout::LayoutType;
use crate::xserver::XConn;
use crate::window::Screen;

pub const MAX_WKSPACES: usize = 10;

#[derive(Clone, Default)]
pub struct Desktop {
    workspaces: Vec<Workspace>,
    current: usize,
}

#[allow(dead_code)]
impl Desktop {
    pub fn new(layout: LayoutType) -> Self {
        Self {
            workspaces: {
                let mut vec = Vec::with_capacity(MAX_WKSPACES);

                for _ in 0..MAX_WKSPACES {
                    vec.push(Workspace::with_layout(layout));
                }

                vec
            },
            current: 0,
        }
    }

    pub fn current(&self) -> &Workspace {
        &self.workspaces[self.current]
    }

    pub fn current_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.current]
    }

    pub fn contains(&mut self, window: XWindowID) -> Option<(&Workspace, usize)> {
        for ws in &self.workspaces {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx))
            }
        }

        None
    }

    pub fn contains_mut(&mut self, window: XWindowID) -> Option<(&mut Workspace, usize)> {
        for ws in &mut self.workspaces {
            if let Some(idx) = ws.contains(window) {
                return Some((ws, idx))
            }
        }

        None
    }

    pub fn get(&self, idx: usize) -> Option<&Workspace> {
        if idx + 1 >= self.workspaces.len() {
            return None
        }

        Some(&self.workspaces[idx])
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Workspace> {
        if idx + 1 >= self.workspaces.len() {
            return None
        }

        Some(&mut self.workspaces[idx])
    }

    pub fn goto(&mut self, conn: &XConn, scr: &Screen, idx: usize) {
        debug!("Goto desktop {}", idx);

        self.workspaces.get_mut(self.current).unwrap().deactivate(conn);
        
        self.current = idx;

        if let Some(ws) = self.get_mut(self.current) {
            ws.activate(conn, scr);
        } else {
            error!("No workspace found for index {}", idx);
        }
    }
}