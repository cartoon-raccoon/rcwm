#![allow(dead_code)]
use std::ops::{Index, IndexMut};
use std::collections::{
    VecDeque, 
    HashSet, 
};

use crate::xserver::{
    XConn,
    XWindow, 
    XWindowID
};
use crate::values;
use crate::types::WindowState;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl From<(i32, i32, i32, i32)> for Geometry {
    fn from(from: (i32, i32, i32, i32)) -> Self {
        Self {
            x: from.0,
            y: from.1,
            width: from.2,
            height: from.3,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct Window {
    pub xwindow: XWindow,
    pub state: WindowState,
    protocols: HashSet<xcb::Atom>,
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.xwindow.id == other.xwindow.id
    }
}

impl From<XWindowID> for Window {
    fn from(from: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(from),
            state: WindowState::Floating,
            protocols: HashSet::new(),
        }
    }
}

//todo: fix your calculations, they are deeply broken.
impl Window {
    pub fn tiled(from: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(from),
            state: WindowState::Tiled,
            protocols: HashSet::new(),
        }
    }

    pub fn floating(from: XWindowID) -> Self {
        Self {
            xwindow: XWindow::from(from),
            state: WindowState::Floating,
            protocols: HashSet::new(),
        }
    }

    #[inline]
    pub fn is_tiled(&self) -> bool {
        if let WindowState::Tiled = self.state {
            return true
        }
        false
    }

    #[inline]
    pub fn is_floating(&self) -> bool {
        if let WindowState::Floating = self.state {
            return true
        }
        false
    }

    #[inline]
    pub fn set_tiled(&mut self) {
        self.state = WindowState::Tiled
    }

    #[inline]
    pub fn set_floating(&mut self) {
        self.state = WindowState::Floating
    }

    #[inline]
    pub fn toggle_state(&mut self) {
        if let WindowState::Floating = self.state {
            self.state = WindowState::Tiled
        } else if let WindowState::Tiled = self.state {
            self.state = WindowState::Floating
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
    
    /// Configure the window using a provided connection
    /// 
    /// Use `Window::set_geometry` and `Window::update_geometry`
    /// to change window geometry instead of this method.
    pub fn configure(&self, conn: &XConn, attrs: &[(u16, u32)]) {
        conn.configure_window(self.id(), attrs);
    }

    /// Change window attributes.
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

    /// Updates and sets the window geometry with a given Geometry.
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
    windows: VecDeque<Window>,
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
    
    pub fn push(&mut self, window: Window) {
        self.windows.push_front(window)
    }

    pub fn pop(&mut self, idx: usize) -> Window {
        self.move_front(idx);

        self.windows.pop_front().unwrap()
    }

    pub fn insert(&mut self, idx: usize, window: Window) {
        self.windows.insert(idx, window)
    }

    pub fn get(&self, idx: usize) -> Option<&Window> {
        self.windows.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Window> {
        self.windows.get_mut(idx)
    }

    pub fn remove(&mut self, idx: usize) -> Option<Window> {
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

    pub fn iter(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Window> {
        self.windows.iter_mut()
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter().rev()
    }

    pub fn set_focused(&mut self, id: XWindowID) {
        if let Some(_) = self.contains(id) {
            self.focused = Some(id)
        } else {
            error!("Tried to focus a window not in the workspace")
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

    pub fn focused(&self) -> Option<&Window> {
        if let Some(win) = self.focused {
            return self.lookup(win)
        }

        None
    }

    pub fn focused_mut(&mut self) -> Option<&mut Window> {
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

    pub fn lookup(&self, id: XWindowID) -> Option<&Window> {
        if let Some(idx) = self.contains(id) {
            return self.get(idx)
        }

        None
    }

    pub fn lookup_mut(&mut self, id: XWindowID) -> Option<&mut Window> {
        if let Some(idx) = self.contains(id) {
            return self.get_mut(idx)
        }

        None
    }
}

impl Index<usize> for Windows {
    type Output = Window;
    
    fn index(&self, idx: usize) -> &Window {
        &self.windows[idx]
    }
}

impl IndexMut<usize> for Windows {
    fn index_mut(&mut self, idx: usize) -> &mut Window {
        &mut self.windows[idx]
    }
} 