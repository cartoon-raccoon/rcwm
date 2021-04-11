use crate::x::core::{XEvent, XWindowID};
use crate::core::types::{Geometry, Keybind};

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
    /// Map the specified client.
    MapClient(XWindowID),
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

