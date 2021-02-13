use crate::workspace::Workspace;

pub const MAX_WKSPACES: usize = 10;

pub struct Desktop {
    workspaces: [Workspace; MAX_WKSPACES],
    current: usize,
}

impl Desktop {

}