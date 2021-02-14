use crate::workspace::Workspace;
use crate::xserver::XWindowID;
use crate::window::Window;

pub const MAX_WKSPACES: usize = 10;

#[derive(Clone, Default)]
pub struct Desktop {
    workspaces: Vec<Workspace>,
    current: usize,
}

#[allow(dead_code)]
impl Desktop {
    pub fn new() -> Self {
        Self {
            workspaces: {
                let mut vec = Vec::with_capacity(MAX_WKSPACES);

                for _ in 0..MAX_WKSPACES {
                    vec.push(Workspace::default());
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

    pub fn contains(&mut self, window: XWindowID) -> Option<&Window> {
        for ws in &self.workspaces {
            if let Some(idx) = ws.contains(window) {
                return Some(&ws[idx])
            }
        }

        None
    }
}