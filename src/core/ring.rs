#![allow(dead_code)]
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use crate::window::Client;
use crate::x::XWindowID;

/// Direction to traverse the ring
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum InsertPoint {
    Index(usize),
    Focused,
    AfterFocused,
    BeforeFocused,
    First,
    Last,
}

pub enum Selector<'a, T> {
    Any,
    Focused,
    Index(usize),
    WindowID(XWindowID),
    Condition(&'a dyn Fn(&T) -> bool),
}

#[derive(Debug, Clone, Default)]
pub struct Ring {
    /// Internal storage of windows
    windows: VecDeque<Client>,
    /// Idx of focused window.
    focused: Option<usize>,
}

impl Ring {

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Whether moving in the given direction would wrap to the front or back.
    #[inline]
    fn would_wrap(&self, direction: Direction) -> bool {
        use Direction::*;

        match direction {
            Forward => {
                if let Some(i) = self.focused {
                    if i == self.len() - 1 {
                        return true
                    }
                }
            }
            Backward => {
                if let Some(i) = self.focused {
                    if i == 0 {
                        return true
                    }
                }
            }
        }
        false
    }

    pub fn move_front(&mut self, idx: usize) {
        if idx != 0 {self.windows.swap(0, idx)}
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

    /// Insert an item into the Ring with an insert point
    /// 
    /// If insert point revolves around the focused item and nothing has focus,
    /// it appends the item to the end of the ring.
    pub fn insert(&mut self, point: InsertPoint, item: Client) {
        use InsertPoint::*;
        match point {
            Index(idx) => {
                self.windows.insert(idx, item);
            }
            Focused => {
                if let Some(idx) = self.focused {
                    self.windows.insert(idx, item);
                } else {
                    self.append(item);
                }
            }
            AfterFocused => {
                if let Some(idx) = self.focused {
                    self.windows.insert(idx + 1, item);
                } else {
                    self.append(item);
                }
            }
            BeforeFocused => {
                if let Some(idx) = self.focused {
                    let idx = if idx == 0 {
                        self.windows.len() - 1
                    } else {
                        idx - 1
                    };
                    self.windows.insert(idx, item);
                } else {
                    self.append(item);
                }
            }
            First => {
                self.windows.insert(0, item);
            }
            Last => {
                self.windows.insert(self.windows.len() - 1, item);
            }
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<Client> {
        self.windows.remove(idx)
    }

    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<Client> {
        if let Some(i) = self.get_idx(id) {
            self.remove(i)
        } else {
            None
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Client> {
        self.windows.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Client> {
        self.windows.get_mut(idx)
    }

    pub fn get_idx(&self, id: XWindowID) -> Option<usize> {
        for (i, win) in self.windows.iter().enumerate() {
            if win.id() == id {
                return Some(i)
            }
        }
        None
    }

    pub fn lookup(&self, id: XWindowID) -> Option<&Client> {
        if let Some(i) = self.get_idx(id) {
            self.get(i)
        } else {
            None
        }
    }

    pub fn lookup_mut(&mut self, id: XWindowID) -> Option<&mut Client> {
        if let Some(i) = self.get_idx(id) {
            self.get_mut(i)
        } else {
            None
        }
    }

    pub fn contains(&mut self, id: XWindowID) -> bool {
        for win in self.windows.iter() {
            if win.id() == id {
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

    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = &mut Client> {
        self.windows.iter_mut().rev()
    }

    pub fn rotate(&mut self, direction: Direction) {
        self.rotate_by(1, direction);
    }

    pub fn rotate_by(&mut self, step: usize, direction: Direction) {
        use Direction::*;
        match direction {
            Forward => {
                self.windows.rotate_right(step);
            }
            Backward => {
                self.windows.rotate_left(step);
            }
        }
    }

    /// Cycles the focus by one in the given direction.
    /// 
    /// Is a no-op if nothing is in focus.
    pub fn cycle_focus(&mut self, direction: Direction) {
        use Direction::*;

        match direction {
            Forward => {
                if let Some(i) = self.focused {
                    if self.would_wrap(Forward) {
                        self.focused = Some(0)
                    } else {
                        self.focused = Some(i + 1)
                    }
                }
            }
            Backward => {
                if let Some(i) = self.focused {
                    if self.would_wrap(Backward) {
                        self.focused = Some(self.len() - 1)
                    } else {
                        self.focused = Some(i - 1)
                    }
                }
            }
        }
    }

    pub fn set_focused(&mut self, id: XWindowID) {
        if let Some(i) = self.get_idx(id) {
            self.focused = Some(i)
        } else {
            error!("Tried to focus a client not in the workspace")
        }
    }

    pub fn set_focused_by_idx(&mut self, idx: usize) {
        self.focused = Some(idx)
    }

    #[inline(always)]
    pub fn unset_focused(&mut self) {
        self.focused = None
    }

    pub fn focused(&self) -> Option<&Client> {
        if let Some(i) = self.focused {
            return self.get(i)
        }

        None
    }

    pub fn focused_mut(&mut self) -> Option<&mut Client> {
        if let Some(i) = self.focused {
            return self.get_mut(i)
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
}

impl Index<usize> for Ring {
    type Output = Client;
    
    fn index(&self, idx: usize) -> &Client {
        &self.windows[idx]
    }
}

impl IndexMut<usize> for Ring {
    fn index_mut(&mut self, idx: usize) -> &mut Client {
        &mut self.windows[idx]
    }
} 