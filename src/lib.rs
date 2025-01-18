/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/sparse-slot
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum SparseSlotError {
    IndexOutOfBounds(usize),
    Occupied(usize),
    //    GenerationMismatch(u8),
    IllegalZeroGeneration,
}

/// A fixed-size sparse collection that maintains optional values at specified indices.
///
/// `SparseSlot<T>` provides a fixed-capacity container where each slot can either be empty (`None`)
/// or contain a value (`Some(T)`). Once initialized, the capacity cannot be changed. Values can only
/// be set once in empty slots - attempting to overwrite an existing value will be ignored.
///
/// # Type Parameters
///
/// * `T` - The type of elements stored in the collection
///
/// # Characteristics
///
/// * Fixed size - Capacity is determined at creation
/// * Sparse storage - Slots can be empty or filled
/// * One-time assignment - Values can only be set once per slot
/// * Index-based access - Direct access to elements via indices
/// * Iterator support - Both immutable and mutable iteration over non-empty slots
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct Id {
    pub index: usize,
    pub generation: u8,
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>4}:{:04X}", self.index, self.generation)
    }
}

impl Id {
    #[must_use]
    pub fn new(index: usize, generation: u8) -> Self {
        Self { index, generation }
    }

    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    pub fn generation(&self) -> u8 {
        self.generation
    }

    #[must_use]
    pub fn next(&self) -> Self {
        Self {
            index: self.index,
            generation: self.generation.wrapping_add(1),
        }
    }
}

impl From<((usize, u8),)> for Id {
    fn from(((index, generation),): ((usize, u8),)) -> Self {
        Self { index, generation }
    }
}

pub struct Iter<'a, T> {
    items: &'a [Entry<T>],
    next_index: Option<usize>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Id, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.next_index?;
        let entry = &self.items[current_index];

        self.next_index = entry.next_index;

        entry
            .item
            .as_ref()
            .map(|item| (Id::new(current_index, entry.generation), item))
    }
}

pub struct IterMut<'a, T> {
    items: &'a mut [Entry<T>],
    next_index: Option<usize>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Id, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.next_index?;

        let entry = unsafe { &mut *(self.items.get_unchecked_mut(current_index) as *mut Entry<T>) };

        let next = entry.next_index;
        self.next_index = next;

        entry
            .item
            .as_mut()
            .map(|item| (Id::new(current_index, entry.generation), item))
    }
}

pub struct IntoIter<T> {
    items: Vec<Entry<T>>,
    next_index: Option<usize>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = (Id, T);

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.next_index?;
        let entry = &mut self.items[current_index];

        // Store next index before taking the item
        let next = entry.next_index;
        self.next_index = next;

        entry
            .item
            .take()
            .map(|item| (Id::new(current_index, entry.generation), item))
    }
}

impl<T> FromIterator<(Id, T)> for SparseSlot<T> {
    fn from_iter<I: IntoIterator<Item = (Id, T)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut slot = Self::new(lower.max(16)); // Default minimum capacity

        for (id, value) in iter {
            let _ = slot.try_set(id, value);
        }
        slot
    }
}

impl<T> IntoIterator for SparseSlot<T> {
    type Item = (Id, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            items: self.items,
            next_index: self.first_occupied,
        }
    }
}

pub struct Keys<'a, T> {
    iter: Iter<'a, T>,
}

pub struct Values<'a, T> {
    iter: Iter<'a, T>,
}

pub struct ValuesMut<'a, T> {
    iter: IterMut<'a, T>,
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(id, _)| id)
    }
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, value)| value)
    }
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, value)| value)
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Entry<T> {
    pub generation: u8,
    pub item: Option<T>,
    pub next_index: Option<usize>,
    pub previous_index: Option<usize>,
}

impl<T> Default for Entry<T> {
    fn default() -> Self {
        Self {
            generation: 0,
            item: None,
            next_index: None,
            previous_index: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SparseSlot<T> {
    items: Vec<Entry<T>>,
    first_occupied: Option<usize>,
}

impl<T> SparseSlot<T> {
    /// Creates a new `SparseSlot` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The fixed size of the collection
    ///
    /// # Returns
    ///
    /// A new `SparseSlot` instance with all slots initialized to `None`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sparse_slot::SparseSlot;
    /// let slot: SparseSlot<i32> = SparseSlot::new(5);
    /// assert_eq!(slot.len(), 0);
    /// assert_eq!(slot.capacity(), 5);
    /// ```
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let mut items = Vec::with_capacity(capacity);
        items.extend((0..capacity).map(|_| Entry::default()));
        Self {
            items,
            first_occupied: None,
        }
    }

    // Mutation ------------------------------------------------------------------------------------

    pub fn try_set(&mut self, id: Id, item: T) -> Result<(), SparseSlotError> {
        if id.index >= self.items.len() {
            return Err(SparseSlotError::IndexOutOfBounds(id.index));
        }

        // First, validate the entry
        {
            let entry = &self.items[id.index];
            if entry.item.is_some() {
                return Err(SparseSlotError::Occupied(id.index));
            }
            if entry.generation != id.generation {
                // return Err(SparseSlotError::GenerationMismatch(entry.generation));
            }
        }

        let mut prev_index = None;
        let mut next_index = self.first_occupied;

        while let Some(current) = next_index {
            if current > id.index {
                break;
            }
            prev_index = Some(current);
            next_index = self.items[current].next_index;
        }

        {
            let entry = &mut self.items[id.index];
            entry.item = Some(item);
            entry.generation = id.generation;
            entry.previous_index = prev_index;
            entry.next_index = next_index;
        }

        if let Some(prev_idx) = prev_index {
            self.items[prev_idx].next_index = Some(id.index);
        } else {
            self.first_occupied = Some(id.index);
        }

        if let Some(next_idx) = next_index {
            self.items[next_idx].previous_index = Some(id.index);
        }

        Ok(())
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let (prev_index, next_index) = {
            let entry = &self.items[id.index];
            if entry.generation != id.generation || entry.item.is_none() {
                return None;
            }
            (entry.previous_index, entry.next_index)
        };

        if Some(id.index) == self.first_occupied {
            self.first_occupied = next_index;
        }

        if let Some(prev_idx) = prev_index {
            self.items[prev_idx].next_index = next_index;
        }
        if let Some(next_idx) = next_index {
            self.items[next_idx].previous_index = prev_index;
        }

        let entry = &mut self.items[id.index];
        let item = entry.item.take();
        entry.generation = entry.generation.wrapping_add(1);
        entry.next_index = None;
        entry.previous_index = None;

        item
    }

    pub fn clear(&mut self) {
        for entry in &mut self.items {
            if entry.item.take().is_some() {
                entry.generation = entry.generation.wrapping_add(1);
                entry.next_index = None;
                entry.previous_index = None;
            }
        }
        self.first_occupied = None;
    }

    // Mutation getters ------------------------------------------------------------------------------------

    pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
        ValuesMut {
            iter: self.iter_mut(),
        }
    }

    #[must_use]
    #[inline(always)]
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        let entry = self.items.get_mut(id.index)?;
        if entry.generation != id.generation {
            return None;
        }
        entry.item.as_mut()
    }

    // Iterators ------------------------------------------------------------------------------------
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            items: &self.items,
            next_index: self.first_occupied,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let first = self.first_occupied;
        IterMut {
            items: &mut self.items,
            next_index: first,
        }
    }

    pub fn keys(&self) -> Keys<'_, T> {
        Keys { iter: self.iter() }
    }

    pub fn values(&self) -> Values<'_, T> {
        Values { iter: self.iter() }
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (Id, T)> + '_ {
        let mut index = self.first_occupied;
        std::iter::from_fn(move || {
            while let Some(current_index) = index {
                let entry = &mut self.items[current_index];
                index = entry.next_index;

                if let Some(item) = entry.item.take() {
                    entry.generation = entry.generation.wrapping_add(1);
                    return Some((Id::new(current_index, entry.generation - 1), item));
                }
            }
            None
        })
    }

    // Query ------------------------------------------------------------------------------------

    pub fn capacity(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.iter().filter(|x| x.item.is_some()).count()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn first_id(&self) -> Option<Id> {
        self.first_occupied.map(|index| {
            let entry = &self.items[index];
            Id::new(index, entry.generation)
        })
    }

    // TODO: This is not efficient, should have a self.last_occupied in the future
    pub fn last_id(&self) -> Option<Id> {
        self.items
            .iter()
            .enumerate()
            .rev()
            .find(|(_, entry)| entry.item.is_some())
            .map(|(index, entry)| Id::new(index, entry.generation))
    }

    // Getters ------------------------------------------------------------------------------------
    #[must_use]
    #[inline(always)]
    pub fn get(&self, id: Id) -> Option<&T> {
        let entry = &self.items[id.index];
        if entry.generation != id.generation {
            return None;
        }
        entry.item.as_ref()
    }
}
