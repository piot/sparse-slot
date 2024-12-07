/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/swamp-render
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */
pub mod prelude;

use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
pub enum SparseSlotError {
    IndexOutOfBounds(usize),
    Occupied(usize),
    GenerationMismatch(u16),
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
    pub generation: u16,
}

impl Id {
    #[must_use]
    pub fn new(index: usize, generation: u16) -> Self {
        Self { index, generation }
    }

    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    pub fn generation(&self) -> u16 {
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

impl From<((usize, u16),)> for Id {
    fn from(((index, generation),): ((usize, u16),)) -> Self {
        Self { index, generation }
    }
}

pub struct Iter<'a, T> {
    items: std::slice::Iter<'a, Entry<T>>,
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Id, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.items.next()?;
            let current_index = self.index;
            self.index += 1;

            if let Some(ref item) = entry.item {
                return Some((Id::new(current_index, entry.generation), item));
            }
        }
    }
}

pub struct IterMut<'a, T> {
    items: std::slice::IterMut<'a, Entry<T>>,
    index: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Id, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.items.next()?;
            let current_index = self.index;
            self.index += 1;

            if let Some(ref mut item) = entry.item {
                return Some((Id::new(current_index, entry.generation), item));
            }
        }
    }
}

#[derive(Debug)]
struct Entry<T> {
    pub generation: u16,
    pub item: Option<T>,
}

impl<T> Default for Entry<T> {
    fn default() -> Self {
        Self {
            generation: 0,
            item: None,
        }
    }
}

#[derive(Debug)]
pub struct SparseSlot<T> {
    items: Vec<Entry<T>>,
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
        Self { items }
    }

    pub fn try_set(&mut self, id: Id, item: T) -> Result<(), SparseSlotError> {
        if id.index >= self.items.len() {
            return Err(SparseSlotError::IndexOutOfBounds(id.index));
        }

        let entry = self
            .items
            .get_mut(id.index)
            .ok_or(SparseSlotError::IndexOutOfBounds(id.index))?;

        if entry.item.is_some() {
            return Err(SparseSlotError::Occupied(id.index));
        }

        if entry.generation != id.generation {
            return Err(SparseSlotError::GenerationMismatch(entry.generation));
        }

        entry.item = Some(item);
        Ok(())
    }

    #[must_use]
    #[inline(always)]
    pub fn get(&self, id: Id) -> Option<&T> {
        let entry = &self.items[id.index];
        if entry.generation != id.generation {
            return None;
        }
        entry.item.as_ref()
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

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let entry = self.items.get_mut(id.index)?;
        if entry.generation != id.generation {
            return None;
        }
        let item = entry.item.take();
        if item.is_some() {
            entry.generation = entry.generation.wrapping_add(1);
        }
        item
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            items: self.items.iter(),
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            items: self.items.iter_mut(),
            index: 0,
        }
    }

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

    pub fn clear(&mut self) {
        for entry in &mut self.items {
            if entry.item.take().is_some() {
                entry.generation = entry.generation.wrapping_add(1);
            }
        }
    }
}
