//! The base functionality for communicating with the X Server.
//! Also contains some extra types that are intrinsic to the X server.

use xcb_util::{ewmh, cursor};
use xcb_util::keysyms::KeySymbols;
use anyhow::{Context, Result};

use std::convert::TryFrom;

use crate::window::Client;
use crate::utils;
use crate::types::{Geometry, Point};

pub use super::{core::*, event::*};

// used for casting events and stuff
macro_rules! cast {
    ($etype:ty, $event:expr) => {
        unsafe {xcb::cast_event::<$etype>(&$event)}
    };
}

/// Holds a handle to an XCB connection, as well as other attributes
/// about the window manager.
pub struct XCBConnection {
    pub(crate) conn: ewmh::Connection,
    pub atoms: InternedAtoms,
    pub root: XWindowID,
    pub cursor: xcb::Cursor,
    pub current_scr: i32,
}

impl XCBConnection {
    /// Creates a new X Connection.
    pub fn new(xconn: ewmh::Connection, idx: i32) -> Self {
        let atoms = InternedAtoms::new(&xconn);
        let root = xconn.get_setup()
            .roots()
            .nth(idx as usize)
            .expect("Could not get root id")
            .root();
        Self {
            conn: xconn,
            atoms: atoms,
            root: root,
            cursor: 0,
            current_scr: idx,
        }
    }

    /// Get a reference to the underlying XCB Connection.
    pub fn get_raw(&self) -> &ewmh::Connection {
        &self.conn
    }
    
    /// Gets the setup of the underlying xcb connection.
    pub fn get_setup(&self) -> xcb::Setup {
        self.conn.get_setup()
    }

    /// Get the root ID of the current screen.
    pub fn get_root_id(&self) -> XWindowID {
        self.conn.get_setup()
            .roots()
            .nth(self.current_scr as usize)
            .expect("Could not get root id")
            .root()
    }

    /// Get the root geometry (which is usually the screen resolution)
    pub fn get_root_geom(&self) -> Result<Geometry> {
        let root_id = self.get_root_id();

        self.get_geometry(root_id)
    }

    /// Set the screen of the current root.
    pub fn set_root_scr(&mut self, scr: i32) {
        self.current_scr = scr;
    }

    /// Get a list of all active windows.
    pub fn query_tree(&self, window: XWindowID) -> Result<Vec<XWindowID>> {
        xcb::query_tree(&self.conn, window).get_reply()
            .map(|ok| ok.children().to_owned())
            .map_err(|err| anyhow::Error::new(err))
    }

    /// Create the cursor.
    pub fn create_cursor(&mut self, glyph: u16) -> Result<()> {
        debug!("Creating cursor");
        let cursor_id = cursor::create_font_cursor_checked(&self.conn, glyph)?;
        self.cursor = cursor_id;
        Ok(())
    }

    /// Set cursor.
    pub fn set_cursor(&mut self, window: XWindowID) {
        debug!("Setting cursor for {}", window);
        self.change_window_attributes(window, &utils::cursor_attrs(self.cursor))
    }

    /// Get window attributes
    pub fn get_window_attributes(&self, window: XWindowID) -> Option<xcb::GetWindowAttributesReply> {
        debug!("Getting attributes for window {}", window);

        xcb::get_window_attributes(&self.conn, window).get_reply().ok()
    }

    pub fn change_window_attributes(&self, window: XWindowID, attrs: &[(u32, u32)]) {
        //debug!("Changing attributes for window {}", window);
        xcb::change_window_attributes(&self.conn, window, attrs);
    }

    pub fn change_window_attributes_checked(&self, window: XWindowID, attrs: &[(u32, u32)]) -> Result<()> {
        //debug!("Changing window attributes");
        xcb::change_window_attributes_checked(&self.conn, window, attrs).request_check()
            .with_context(|| String::from("Could not change window attributes"))
    }

    pub fn configure_window(&self, window: XWindowID, attrs: &[(u16, u32)]) {
        debug!("Configuring window {}", window);
        xcb::configure_window(&self.conn, window, attrs);
    }

    pub fn reparent_window(&self, window: XWindowID, parent: XWindowID) {
        debug!("Reparenting window {} under window {}", window, parent);
        xcb::reparent_window(&self.conn, window, parent, 0, 0);
    }

    // pub fn create_window(&self, parent: XWindowID, geom: Geometry, border_width,) -> XWindowID {
    //     let win_id = self.conn.generate_id();

    //     let cookie = xcb::create_window(
    //         &self.conn,
    //         xcb::COPY_FROM_PARENT,
    //         win_id,
    //         parent,
    //         geom.x,
    //         geom.y,
    //         geom.height,
    //         geom.width,
    //     );
    // }

    pub fn map_window(&self, window_id: XWindowID) {
        debug!("Mapping window {}", window_id);

        let cookie = xcb::map_window(&self.conn, window_id);
        if let Err(e) = cookie.request_check() {
            error!("Could not map window {}: {}", window_id, e)
        }
    }

    pub fn unmap_window(&self, window_id: XWindowID) {
        debug!("Unmapping window {}", window_id);

        let cookie = xcb::unmap_window(&self.conn, window_id);
        if let Err(e) = cookie.request_check() {
            error!("Could not unmap window {}: {}", window_id, e)
        }
    }

    pub fn destroy_window(&self, window: &Client) {
        debug!("Destroying window {}", window.id());
        if window.supports(self.atoms.WM_DELETE_WINDOW) {
            debug!("Destroying window via ICCCM WM_DELETE_WINDOW");

            let msg_data = xcb::ClientMessageData::from_data32([
                self.atoms.WM_DELETE_WINDOW,
                xcb::CURRENT_TIME,
                0, 0, 0 
            ]);

            let msg_event = xcb::ClientMessageEvent::new(32, window.id(), self.atoms.WM_PROTOCOLS, msg_data);

            xcb::send_event(
                &self.conn,
                false,
                window.id(),
                xcb::EVENT_MASK_NO_EVENT,
                &msg_event
            );
        } else {
            debug!("Destroying window via xcb::destroy_window");

            xcb::destroy_window(&self.conn, window.id());
        }
    }

    pub fn set_input_focus(&self, window_id: XWindowID) {
        debug!("Setting focus for window {}", window_id);

        xcb::set_input_focus(&self.conn, xcb::INPUT_FOCUS_POINTER_ROOT as u8, window_id, xcb::CURRENT_TIME);
    }

    pub fn set_geometry(&self, window_id: XWindowID, geom :Geometry) {
        self.configure_window(window_id, &utils::configure_resize(
            geom.width as u32,
            geom.height as u32,
        ));

        self.configure_window(window_id, &utils::configure_move(
            geom.x as u32,
            geom.y as u32,
        ))
    } 

    pub fn get_geometry(&self, window_id: XWindowID) -> Result<Geometry> {
        debug!("Getting geometry");
        xcb::get_geometry(&self.conn, window_id).get_reply()
            .with_context(|| {
                String::from("Failed to get window geometry")
            })
            .map(|ok| Geometry::from(
                (
                    ok.x() as i32, 
                    ok.y() as i32, 
                    ok.width() as i32, 
                    ok.height() as i32,
                )
            ))
    }

    pub fn lookup_keysym(&self, event:&xcb::KeyPressEvent) -> (xcb::ModMask, xcb::Keysym) {
        let keysym = KeySymbols::new(&self.conn).press_lookup_keysym(event, 0);

        (event.state() as u32, keysym)
    }

    pub fn grab_key(&self, window_id: XWindowID, mask: xcb::ModMask, keysym: xcb::Keysym) {
        debug!("Grabbing key {} for window {}", keysym, window_id);

        let code = KeySymbols::new(&self.conn).get_keycode(keysym).next();

        if let Some(code) = code {
            xcb::grab_key(
                &self.conn,
                false,
                window_id,
                mask as u16,
                code,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            ).request_check().unwrap_or_else(|_e| {
                warn!("Unable to grab key {}", keysym)
            });
        } else {
            warn!("Returned null keycode for keysym {}, not grabbing", keysym)
        }
    }

    pub fn grab_button(&self, 
        window_id: XWindowID, 
        mask: xcb::ButtonMask, 
        button: xcb::ButtonIndex, 
        modmask: xcb::ModMask, 
        confine: bool) 
    {
        debug!("Grab button {} for window: {}", button, window_id);

        xcb::grab_button(
            &self.conn, 
            false, 
            window_id, 
            mask as u16, 
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            if confine { window_id } else { xcb::NONE },
            xcb::NONE,
            button as u8,
            modmask as u16,
        ).request_check().unwrap_or_else(|_e| {
            warn!("Unable to grab button {}", button)
        });
    }

    pub fn grab_pointer(&self, window_id: XWindowID, mask: xcb::EventMask) {
        debug!("Grabbing pointer for window: {}", window_id);

        xcb::grab_pointer(
            &self.conn,
            false,
            window_id,
            mask as u16,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::NONE,
            xcb::NONE,
            xcb::CURRENT_TIME,
        );
    }

    pub fn ungrab_pointer(&self) {
        debug!("Ungrabbing pointer");

        xcb::ungrab_pointer(&self.conn, xcb::CURRENT_TIME);
    }

    pub fn query_pointer(&self, window_id: XWindowID) -> Result<xcb::QueryPointerReply> {
        debug!("Querying pointer location for window {}", window_id);

        xcb::query_pointer(&self.conn, window_id).get_reply()
        .with_context(|| String::from("Failed to query pointer"))
    }

    pub fn next_event(&self) -> xcb::GenericEvent {
        self.conn.flush();

        if let Some(event) = self.conn.poll_for_queued_event() {
            event
        } else {
            self.conn.wait_for_event().unwrap_or_else(|| {
                error!("The X server closed the connection, exiting");
                std::process::exit(10);
            })
        }
    }

    pub fn process_raw_event(&self, event: xcb::GenericEvent) -> XEvent {
        use XEvent::*;
        #[allow(unused_imports)]
        use xcb:: {
            ConfigureNotifyEvent,
            ConfigureRequestEvent,
            MapRequestEvent,
            MapNotifyEvent,
            UnmapNotifyEvent,
            DestroyNotifyEvent,
            EnterNotifyEvent,
            LeaveNotifyEvent,
            MotionNotifyEvent,
            ReparentNotifyEvent,
            PropertyNotifyEvent,
            KeyPressEvent,
            ButtonPressEvent,
            ClientMessageEvent,
        };
        match event.response_type() & !0x80 {
            xcb::CONFIGURE_NOTIFY => {
                let event = cast!(ConfigureNotifyEvent, event);

                ConfigureNotify(ConfigureEvent{
                    id: event.window(),
                    geom: Geometry {
                        x: event.x() as i32,
                        y: event.y() as i32,
                        width: event.width() as i32,
                        height: event.height() as i32,
                    },
                    is_root: event.window() == self.root,
                })
            }
            xcb::CONFIGURE_REQUEST => {
                let event = cast!(ConfigureRequestEvent, event);

                ConfigureRequest(ConfigureEvent{
                    id: event.window(),
                    geom: Geometry {
                        x: event.x() as i32,
                        y: event.y() as i32,
                        width: event.width() as i32,
                        height: event.height() as i32,
                    },
                    is_root: event.window() == self.root,
                })
            },
            xcb::MAP_REQUEST => {
                let event = cast!(MapRequestEvent, event);
                let override_redirect = 
                if let Some(attrs) = self.get_window_attributes(event.window()) {
                    attrs.override_redirect()
                } else {false};

                MapRequest(event.window(), override_redirect)
            },
            xcb::MAP_NOTIFY => {
                let event = cast!(MapNotifyEvent, event);

                MapNotify(event.window())
            },
            xcb::UNMAP_NOTIFY => {
                let event = cast!(UnmapNotifyEvent, event);

                UnmapNotify(event.window())
            },
            xcb::DESTROY_NOTIFY => {
                let event = cast!(DestroyNotifyEvent, event);

                DestroyNotify(event.window())
            },
            xcb::ENTER_NOTIFY => {
                let event = cast!(EnterNotifyEvent, event);

                EnterNotify(event.child())
            },
            xcb::LEAVE_NOTIFY => {
                let event = cast!(LeaveNotifyEvent, event);

                LeaveNotify(event.child())
            },
            xcb::MOTION_NOTIFY => {
                let event = cast!(MotionNotifyEvent, event);

                MotionNotify(Point {
                    x: event.root_x() as i32,
                    y: event.root_y() as i32,
                })
            },
            xcb::REPARENT_NOTIFY => {
                let event = cast!(ReparentNotifyEvent, event);

                ReparentNotify(event.window())
            },
            xcb::PROPERTY_NOTIFY => {
                let event = cast!(PropertyNotifyEvent, event);

                PropertyNotify(PropertyEvent {
                    id: event.window(),
                    atom: event.atom(),
                    time: event.time(),
                })
            },
            xcb::KEY_PRESS => {
                let event = cast!(KeyPressEvent, event);

                let (modmask, keysym) = self.lookup_keysym(event);

                KeyPress(KeypressEvent {
                    mask: modmask,
                    keysym: keysym,
                })
            },
            xcb::KEY_RELEASE => KeyRelease,
            xcb::BUTTON_PRESS => {
                let _event = cast!(ButtonPressEvent, event);

                todo!()
            },
            xcb::BUTTON_RELEASE => ButtonRelease,
            xcb::CLIENT_MESSAGE => {
                let event = cast!(ClientMessageEvent, event);
                let data = event.data();
                let window = event.window();

                match event.format() {
                    8 => {
                        let data = ClientMessageData::try_from(
                            data.data8()
                        ).expect("Invalid client data");
                        assert!(data.is_u8());

                        ClientMessage(window, data)
                    }
                    16 => {
                        let data = ClientMessageData::try_from(
                            data.data16()
                        ).expect("Invalid client data");
                        assert!(data.is_u16());
                        
                        ClientMessage(window, data)
                    }
                    32 => {
                        let data = ClientMessageData::try_from(
                            data.data32()
                        ).expect("Invalid client data");
                        assert!(data.is_u32());
                        
                        ClientMessage(window, data)
                    }
                    _ => unreachable!() //todo: make this function fallible
                }
            },
            unhandled => Unknown(unhandled),
        }
    }
}