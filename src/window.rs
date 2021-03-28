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
use crate::layout::LayoutType;

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

#[derive(Debug, Clone, Copy)]
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

#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    Tiled,
    Floating,
}

impl From<LayoutType> for WindowState {

    #[inline]
    fn from(from: LayoutType) -> WindowState {
        if let LayoutType::Floating = from {
            return Self::Floating
        }

        Self::Tiled
    }
}

#[derive(Clone)]
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

        debug!(
            "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
            self.x(), self.y(), self.height(), self.width()
        );
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

        debug!(
            "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
            self.x(), self.y(), self.height(), self.width()
        );
    }

    // Updates and sets the window geometry with a given Geometry.
    pub fn update_geometry(&mut self, conn: &XConn, geom: Geometry) {
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

#[derive(Default, Clone)]
pub struct Windows {
    windows: VecDeque<Window>,
}

#[allow(dead_code)]
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

    pub fn iter(&mut self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    pub fn iter_rev(&mut self) -> impl Iterator<Item = &Window> {
        self.windows.iter().rev()
    }

    pub fn focused(&self) -> Option<&Window> {
        self.windows.get(0)
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