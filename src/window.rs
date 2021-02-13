use std::ops::{Index, IndexMut};
use std::collections::{
    VecDeque, 
    HashSet, 
    vec_deque::Iter
};

use crate::xserver::{
    XConn,
    XWindow, 
    XWindowID
};

pub const WIN_HEIGHT_MIN: i32 = 20;
pub const WIN_WIDTH_MIN: i32 = 20;

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

pub struct Window {
    pub xwindow: XWindow,
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
            protocols: HashSet::new(),
        }
    }
}

impl Window {
    pub fn id(&self) -> XWindowID {
        self.xwindow.id
    }

    pub fn resize(&mut self, conn: &XConn, ) {

    }
}

#[derive(Default)]
pub struct Windows {
    windows: VecDeque<Window>,
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

    pub fn remove(&mut self, idx: usize) -> Option<Window> {
        self.windows.remove(idx)
    }

    pub fn contains(&self, id: XWindowID) -> Option<usize> {
        let mut idx: usize = 0;

        for window in &self.windows {
            if window.id() == id {
                return Some(idx)
            }

            idx += 1;
        }

        None
    }

    pub fn iter(&mut self) -> WindowsIter<'_> {
        WindowsIter {
            inner: self.windows.iter(),
        }
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

pub struct WindowsIter<'a> {
    inner: Iter<'a, Window>,
}

impl<'a> Iterator for WindowsIter<'a> {
    type Item = &'a Window;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}