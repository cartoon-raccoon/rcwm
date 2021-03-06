use crate::x::{core::{XEvent, XWindowID}, Ewmh};
use crate::core::types::{Geometry, Keybind};
use crate::manager::WMState;

pub enum EventAction {
    /// Focus the specified client.
    ClientFocus(XWindowID),
    /// Unfocus the specified client.
    ClientUnfocus(XWindowID),
    /// Change the WM_NAME property of the specified client.
    ClientNameChange(XWindowID),
    /// Send the focused client to the specified workspace.
    ClientToWorkspace(usize),
    /// Switch to the specified workspace.
    GotoWorkspace(usize),
    /// Destroy the specified client.
    DestroyClient(XWindowID),
    /// Map the specified client and track it internally.
    /// 
    /// Applies to normal windows.
    MapTrackedClient(XWindowID),
    /// Map the specified client and manage it without tracking.
    /// 
    /// Used for dialogue boxes and other windows that have
    /// WM_TRANSIENT_FOR set.
    MapUntrackedClient(XWindowID),
    /// Unmap the specified client.
    UnmapClient(XWindowID),
    /// Configure the specified client with the given geometry.
    ConfigureClient(XWindowID, Geometry),
    /// Run the specified keybind.
    RunKeybind(Keybind),
    /// Toggle the client in or out of fullscreen.
    /// 
    /// Also toggles _NET_WM_STATE_FULLSCREEN.
    ToggleClientFullscreen(XWindowID, bool),
}

impl EventAction {

    #[allow(unused_variables)]
    pub(crate) fn from_xevent(event: XEvent, state: WMState<'_>) -> Vec<Self> {
        use EventAction::*;
        use XEvent::*;
        match event {
            ConfigureNotify(event) | ConfigureRequest(event) => {
                if !event.is_root {
                    vec![ConfigureClient(event.id, event.geom)]
                } else {
                    //todo: add root configure actions
                    vec![]
                }
            },
            MapRequest(id, override_redirect) => {
                if override_redirect {
                    
                }
            },
            MapNotify(id) => vec![],
            UnmapNotify(id) => vec![],
            DestroyNotify(id) => vec![],
            EnterNotify(id) => vec![],
            LeaveNotify(id) => {},
            MotionNotify(pt) => {},
            ReparentNotify(id) => {},
            PropertyNotify(event) => {},
            KeyPress(event) => {},
            KeyRelease => {},
            ButtonPress(id, pt) => {},
            ButtonRelease => {},
            ClientMessage(id, data) => {},
            Unknown(smth) => {},
        }
        todo!()
    }
}

#[allow(unused_imports)]
fn process_map_request(
    id: XWindowID, ovrd: bool, state: WMState<'_>
) -> Vec<EventAction> {
    use EventAction::*;
    use XEvent::*;
    
    if let Some(window_type) = state.conn.get_window_type(id) {
        let atoms = state.conn.get_atoms();


    }

    todo!("process_map_request")
}

