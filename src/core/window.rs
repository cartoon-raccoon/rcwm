#![allow(dead_code)]
use std::ops::{Index, IndexMut};
use std::collections::{
    VecDeque, 
    HashSet, 
};

use xcb_util::icccm;

use crate::x::core::{
    XConn,
    XWindow, 
    XWindowID
};
use crate::x::Icccm;
use crate::values;
use crate::types::{WinLayoutState, Geometry};
use crate::desktop::Screen;

pub const WIN_HEIGHT_MIN: i32 = 100;
pub const WIN_WIDTH_MIN: i32 = 100;
pub const MIN_ONSCREEN: i32 = 20;

fn ensure_in_bounds(val: &mut i32, min: i32, max: i32) {
    if *val < min {
        *val = min;
    } else if *val > max {
        *val = max;
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub xwindow: XWindow,
    pub state: WinLayoutState,
    pub name: String,
    pub icon_name: String,
    
    protocols: HashSet<xcb::Atom>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.xwindow.id == other.xwindow.id
    }
}

//todo: fix your calculations, they are deeply broken.
impl Client {
    pub fn tiled(from: XWindowID, conn: &XConn) -> Self {
        Self::new(from, conn, WinLayoutState::Tiled)
    }

    pub fn floating(from: XWindowID, conn: &XConn) -> Self {
        Self::new(from, conn, WinLayoutState::Floating)
    }

    fn new(from: XWindowID, conn: &XConn, layout: WinLayoutState) -> Self {
        let properties = conn.get_client_properties(from);
        Self {
            xwindow: XWindow::from(from),
            state: layout,
            name: properties.wm_name,
            icon_name: properties.wm_icon_name,
            protocols: HashSet::new(),
        }
    }

    #[inline]
    pub fn is_tiled(&self) -> bool {
        if let WinLayoutState::Tiled = self.state {
            return true
        }
        false
    }

    #[inline]
    pub fn is_floating(&self) -> bool {
        if let WinLayoutState::Floating = self.state {
            return true
        }
        false
    }

    #[inline]
    pub fn set_tiled(&mut self) {
        self.state = WinLayoutState::Tiled
    }

    #[inline]
    pub fn set_floating(&mut self) {
        self.state = WinLayoutState::Floating
    }

    #[inline]
    pub fn toggle_state(&mut self) {
        if let WinLayoutState::Floating = self.state {
            self.state = WinLayoutState::Tiled
        } else if let WinLayoutState::Tiled = self.state {
            self.state = WinLayoutState::Floating
        }
    }

    #[inline(always)]
    pub fn id(&self) -> XWindowID {
        self.xwindow.id
    }

    #[inline(always)]
    pub fn x(&self) -> i32 {
        self.xwindow.geom.x
    }

    #[inline(always)]
    pub fn y(&self) -> i32 {
        self.xwindow.geom.y
    }

    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.xwindow.geom.height
    }

    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.xwindow.geom.width
    }

    pub fn do_move(&mut self, conn: &XConn, _scr: &Screen, dx: i32, dy: i32) {
        self.xwindow.update_pos_y(dy);
        self.xwindow.update_pos_x(dx);

        // let scrx = scr.xwindow.geom.x;
        // let scry = scr.xwindow.geom.y;
        // let scrh = scr.xwindow.geom.height;
        // let scrw = scr.xwindow.geom.width;

        // ensure_in_bounds(&mut self.xwindow.geom.x, 
        //     scrx - self.xwindow.geom.width + MIN_ONSCREEN, 
        //     scrx + scrw - MIN_ONSCREEN);
        // ensure_in_bounds(&mut self.xwindow.geom.y, 
        //     scry - self.xwindow.geom.height + MIN_ONSCREEN, 
        //     scry + scrh - MIN_ONSCREEN);

        conn.configure_window(self.xwindow.id, &values::configure_move(
            self.x() as u32, 
            self.y() as u32
        ));

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }
    
    /// Configure the Client using a provided connection
    /// 
    /// Use `Client::set_geometry` and `Client::update_geometry`
    /// to change Client geometry instead of this method.
    pub fn configure(&self, conn: &XConn, attrs: &[(u16, u32)]) {
        conn.configure_window(self.id(), attrs);
    }

    /// Change Client attributes.
    pub fn change_attributes(&self, conn: &XConn, attrs: &[(u32, u32)]) {
        conn.change_window_attributes(self.id(), attrs)
    }

    pub fn do_resize(&mut self, conn: &XConn, _scr: &Screen, dx: i32, dy: i32) {
        self.xwindow.update_height(dy);
        self.xwindow.update_width(dx);

        // let scrx = scr.xwindow.geom.x;
        // let scry = scr.xwindow.geom.y;
        // let scrh = scr.xwindow.geom.height;
        // let scrw = scr.xwindow.geom.width;

        // ensure_in_bounds(
        //     &mut self.xwindow.geom.height, 
        //     WIN_HEIGHT_MIN, scry + scrh - self.xwindow.geom.y);
        // ensure_in_bounds(&mut self.xwindow.geom.width, 
        //     WIN_WIDTH_MIN, scrx + scrw - self.xwindow.geom.x);

        conn.configure_window(self.xwindow.id, &values::configure_resize(
            self.width() as u32, 
            self.height() as u32
        ));

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }

    pub fn set_geometry(&mut self, geom: Geometry) {
        self.xwindow.set_geometry(geom);
    }

    /// Updates its geometry on the X server.
    pub fn update_geometry(&self, conn: &XConn) {
        conn.configure_window(self.xwindow.id, &values::configure_resize(
            self.width() as u32,
            self.height() as u32,
        ));

        conn.configure_window(self.xwindow.id, &values::configure_move(
            self.x() as u32,
            self.y() as u32,
        ))
    }

    /// Updates and sets the Client geometry with a given Geometry.
    pub fn set_and_update_geometry(&mut self, conn: &XConn, geom: Geometry) {
        self.xwindow.set_geometry(geom);

        conn.configure_window(self.xwindow.id, &values::configure_resize(
            self.width() as u32,
            self.height() as u32,
        ));

        conn.configure_window(self.xwindow.id, &values::configure_move(
            self.x() as u32,
            self.y() as u32,
        ))
    }

    pub fn set_supported(&mut self, conn: &XConn) {
        if let Some(protocols) = conn.get_wm_protocols(self.id()) {
            for protocol in protocols {
                self.protocols.insert(protocol);
            }
        }
    }

    pub fn supports(&self, prtcl: xcb::Atom) -> bool {
        self.protocols.contains(&prtcl)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Windows {
    windows: VecDeque<Client>,
    focused: Option<XWindowID>,
}

impl Windows {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn move_front(&mut self, idx: usize) {
        if idx != 0 { self.windows.swap(0, idx) }
    }
    
    pub fn push(&mut self, window: Client) {
        self.windows.push_front(window)
    }

    pub fn append(&mut self, window: Client) {
        self.windows.push_back(window)
    }

    pub fn pop(&mut self, idx: usize) -> Client {
        self.move_front(idx);

        self.windows.pop_front().unwrap()
    }

    pub fn insert(&mut self, idx: usize, window: Client) {
        self.windows.insert(idx, window)
    }

    pub fn get(&self, idx: usize) -> Option<&Client> {
        self.windows.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Client> {
        self.windows.get_mut(idx)
    }

    pub fn remove(&mut self, idx: usize) -> Option<Client> {
        self.windows.remove(idx)
    }

    pub fn contains(&self, id: XWindowID) -> Option<usize> {

        for (idx, window) in self.windows.iter().enumerate() {
            if window.id() == id {
                return Some(idx)
            }
        }

        None
    }

    pub fn has(&self, id: XWindowID) -> bool {
        for window in self.windows.iter() {
            if window.id() == id {
                return true
            }
        }
        false
    }

    pub fn iter(&self) -> impl Iterator<Item = &Client> {
        self.windows.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.windows.iter_mut()
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &Client> {
        self.windows.iter().rev()
    }

    pub fn set_focused(&mut self, id: XWindowID) {
        if let Some(_) = self.contains(id) {
            self.focused = Some(id)
        } else {
            error!("Tried to focus a Client not in the workspace")
        }
    }

    #[inline(always)]
    pub fn unset_focused(&mut self) {
        self.focused = None
    }

    // Will panic if idx is oob.
    pub fn set_focused_by_idx(&mut self, idx: usize) {
        self.focused = Some(self[idx].id())
    }

    pub fn focused(&self) -> Option<&Client> {
        if let Some(win) = self.focused {
            return self.lookup(win)
        }

        None
    }

    pub fn focused_mut(&mut self) -> Option<&mut Client> {
        if let Some(win) = self.focused {
            return self.lookup_mut(win)
        }

        None
    }

    pub fn is_focused(&self, id: XWindowID) -> bool {
        if let Some(window) = self.focused() {
            return window.id() == id
        } else {
            false
        }
    }

    pub fn lookup(&self, id: XWindowID) -> Option<&Client> {
        if let Some(idx) = self.contains(id) {
            return self.get(idx)
        }

        None
    }

    pub fn lookup_mut(&mut self, id: XWindowID) -> Option<&mut Client> {
        if let Some(idx) = self.contains(id) {
            return self.get_mut(idx)
        }

        None
    }
}

impl Index<usize> for Windows {
    type Output = Client;
    
    fn index(&self, idx: usize) -> &Client {
        &self.windows[idx]
    }
}

impl IndexMut<usize> for Windows {
    fn index_mut(&mut self, idx: usize) -> &mut Client {
        &mut self.windows[idx]
    }
} 