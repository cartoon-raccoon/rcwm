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

/// The point at which to insert the item.
/// 
/// All items at or after the index at which the item 
/// is inserted are shifted backwards.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum InsertPoint {
    /// At the given index.
    /// 
    /// Panics if the index is greater than the len of the Ring.
    Index(usize),
    /// At the index of the focused window.
    /// 
    /// The window is appended if nothing is in focus.
    Focused,
    /// After the index of the focused window.
    AfterFocused,
    /// Before the index of the focused window.
    BeforeFocused,
    First,
    Last,
}

/// Selects a item or a group of items.
pub enum Selector<'a, T> {
    Any,
    Focused,
    Index(usize),
    Condition(&'a dyn Fn(&T) -> bool),
}

/// A Ring of type Client.
///
/// Contains additional methods more specific to window management.
pub type ClientRing = Ring<Client>;

impl ClientRing {

    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<Client> {
        if let Some(i) = self.get_idx(id) {
            self.remove(i)
        } else {
            None
        }
    }

    pub fn get_idx(&self, id: XWindowID) -> Option<usize> {
        self.index(Selector::Condition(&|win| win.id() == id))
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
        for win in self.items.iter() {
            if win.id() == id {
                return true
            }
        }
        false
    }

    pub fn set_focused_by_winid(&mut self, id: XWindowID) {
        if let Some(i) = self.get_idx(id) {
            self.focused = Some(i)
        } else {
            error!("Tried to focus a client not in the workspace")
        }
    }

    pub fn set_focused_by_idx(&mut self, idx: usize) {
        self.set_focused(idx);
    }

    pub fn is_focused(&self, id: XWindowID) -> bool {
        if let Some(window) = self.focused() {
            return window.id() == id
        } else {
            false
        }
    }
}

/// An internal data structure to manage windows internally.
/// 
/// Provides an interface where the data is a ring of items,
/// with a single item in focus.
/// Ideally, the only time there is no focused client is when the ring is empty.
#[derive(Debug, Clone, Default)]
pub struct Ring<T> {
    /// Internal storage of windows
    items: VecDeque<T>,
    /// Idx of focused window.
    focused: Option<usize>,
}

impl<T> Ring<T> {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
            focused: Some(0),
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Whether moving in the given direction would wrap to the front or back.
    /// 
    /// Checks with respect to the focused window.
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
        if idx != 0 {self.items.swap(0, idx)}
    }

    pub fn push(&mut self, item: T) {
        self.items.push_front(item)
    }

    pub fn append(&mut self, item: T) {
        self.items.push_back(item)
    }

    pub fn pop(&mut self, idx: usize) -> T {
        self.move_front(idx);

        self.items.pop_front().unwrap()
    }

    /// Insert an item into the Ring with an insert point
    /// 
    /// If insert point revolves around the focused item and nothing has focus,
    /// it appends the item to the end of the ring.
    pub fn insert(&mut self, point: InsertPoint, item: T) {
        use InsertPoint::*;
        use Direction::*;

        match point {
            Index(idx) => {
                // don't bother checking for whether it would wrap or not
                self.items.insert(idx, item);
            }
            Focused => {
                if let Some(idx) = self.focused {
                    self.items.insert(idx, item);
                } else {
                    self.append(item);
                }
            }
            AfterFocused => {
                if let Some(idx) = self.focused {
                    if self.would_wrap(Forward) {
                        self.items.push_back(item);
                    } else {
                        self.items.insert(idx + 1, item);
                    }
                } else {
                    self.append(item);
                }
            }
            BeforeFocused => {
                if let Some(idx) = self.focused {
                    if self.would_wrap(Backward) {
                        self.items.push_back(item);
                    } else {
                        self.items.insert(idx - 1, item);
                    }
                } else {
                    self.append(item);
                }
            }
            First => {
                self.items.insert(0, item);
            }
            Last => {
                self.items.insert(self.items.len() - 1, item);
            }
        }
    }

    pub fn remove(&mut self, idx: usize) -> Option<T> {
        self.items.remove(idx)
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.items.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.items.get_mut(idx)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub fn iter_rev(&self) -> impl Iterator<Item = &T> {
        self.items.iter().rev()
    }

    pub fn iter_rev_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().rev()
    }

    /// Rotates the entire buffer by 1 in the given direction.
    /// 
    /// Preserves the the focus.
    pub fn rotate(&mut self, direction: Direction) {
        self.rotate_by(1, direction);
        self.cycle_focus(direction);
    }

    pub(crate) fn rotate_by(&mut self, step: usize, direction: Direction) {
        use Direction::*;
        match direction {
            Forward => {
                self.items.rotate_right(step);
            }
            Backward => {
                self.items.rotate_left(step);
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

    #[inline(always)]
    pub fn unset_focused(&mut self) {
        self.focused = None
    }

    pub fn focused(&self) -> Option<&T> {
        if let Some(i) = self.focused {
            return self.get(i)
        }

        None
    }

    pub fn focused_mut(&mut self) -> Option<&mut T> {
        if let Some(i) = self.focused {
            return self.get_mut(i)
        }

        None
    }

    pub fn set_focused(&mut self, idx: usize) {
        self.focused = Some(idx);
    }

    pub fn apply_to<F: FnMut(&mut T)>(&mut self, s: Selector<'_, T>, mut f: F) {
        if let Some(idx) = self.index(s) {
            f(&mut self[idx])
        }
    }

    fn element_by(&self, cond: impl Fn(&T) -> bool) -> Option<(usize, &T)> {
        self.iter().enumerate().find(|(_, e)| cond(*e))
    }

    fn element_by_mut(&mut self, cond: impl Fn(&T) -> bool) -> Option<(usize, &mut T)> {
        self.iter_mut().enumerate().find(|(_, e)| cond(*e))
    }

    pub fn index(&self, s: Selector<'_, T>) -> Option<usize> {
        use Selector::*;

        match s {
            Any | Focused => {
                self.focused
            }
            Index(i) => {
                if i < self.len() {
                    Some(i)
                } else {
                    None
                }
            }
            Condition(f) => {
                self.element_by(f).map(|(i, _)| i)
            }
        }
    }
}

impl<T> Index<usize> for Ring<T> {
    type Output = T;
    
    fn index(&self, idx: usize) -> &T {
        &self.items[idx]
    }
}

impl<T> IndexMut<usize> for Ring<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.items[idx]
    }
} 