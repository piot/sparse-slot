/*
 * Copyright (c) Peter Bjorklund. All rights reserved. https://github.com/piot/sparse-slot
 * Licensed under the MIT License. See LICENSE in the project root for license information.
 */

use sparse_slot::prelude::*;

#[test]
fn basic_operations() {
    let mut slot = SparseSlot::new(3);
    let id = Id::new(1, 0);

    assert!(slot.try_set(id, "hello").is_ok());
    assert_eq!(slot.get(id), Some(&"hello"));
    assert_eq!(slot.remove(id), Some("hello"));
    assert_eq!(slot.get(id), None);
    assert_eq!(slot.len(), 0);
}

#[test]
fn generation_handling() {
    let mut slot = SparseSlot::new(2);
    let id1 = Id::new(1, 0);

    assert!(slot.try_set(id1, 42).is_ok());
    assert_eq!(slot.remove(id1), Some(42));

    // Try to use old generation - should fail
    assert!(slot.try_set(id1, 43).is_err());

    // Use next generation - should succeed
    let id2 = id1.next();
    assert!(slot.try_set(id2, 43).is_ok());
    assert_eq!(slot.get(id2), Some(&43));
}

#[test]
fn error_conditions() {
    let mut slot = SparseSlot::new(1);
    let id = Id::new(0, 0);

    // Test double set
    assert!(slot.try_set(id, 1).is_ok());
    assert!(matches!(
        slot.try_set(id, 2),
        Err(SparseSlotError::Occupied(_))
    ));

    // Test out of bounds
    let invalid_id = Id::new(999, 0);
    assert!(matches!(
        slot.try_set(invalid_id, 3),
        Err(SparseSlotError::IndexOutOfBounds(_))
    ));
}

#[test]
fn iteration() {
    let mut slot = SparseSlot::new(3);
    let id0 = Id::new(1, 0);
    let id2 = Id::new(2, 0);

    slot.try_set(id0, "first").unwrap();
    slot.try_set(id2, "third").unwrap();

    let mut iter_items: Vec<_> = slot.iter().collect();
    iter_items.sort_by_key(|(id, _)| id.index());

    assert_eq!(iter_items.len(), 2);
    assert_eq!(iter_items[0].1, &"first");
    assert_eq!(iter_items[1].1, &"third");

    // Test mutable iteration
    for (_, value) in slot.iter_mut() {
        *value = "changed";
    }

    assert_eq!(slot.get(id0), Some(&"changed"));
    assert_eq!(slot.get(id2), Some(&"changed"));
}

#[test]
fn clear_and_capacity() {
    let mut slot = SparseSlot::new(2);
    let id0 = Id::new(0, 0);
    let id1 = Id::new(1, 0);

    slot.try_set(id0, 1).unwrap();
    slot.try_set(id1, 2).unwrap();

    assert_eq!(slot.len(), 2);
    assert_eq!(slot.capacity(), 2);

    slot.clear();
    assert_eq!(slot.len(), 0);
    assert_eq!(slot.capacity(), 2);

    // Old IDs should no longer work
    assert!(slot.get(id0).is_none());
    assert!(slot.get(id1).is_none());

    // New generations should work
    assert!(slot.try_set(id0.next(), 3).is_ok());
    assert!(slot.try_set(id1.next(), 4).is_ok());
}

// Test iterators

#[test]
fn iterator_ownership() {
    let mut slot = SparseSlot::new(5);

    // Set up some values
    slot.try_set(Id::new(0, 0), "first").unwrap();
    slot.try_set(Id::new(2, 0), "second").unwrap();
    slot.try_set(Id::new(4, 0), "third").unwrap();

    let collected: Vec<_> = slot.into_iter().collect();
    assert_eq!(collected.len(), 3);
    // This would not compile: println!("{:?}", slot);

    let slot: SparseSlot<&str> = SparseSlot::new(5);
    let _iter = slot.iter();
    let _also_slot = &slot; // Can still borrow slot while iterator exists

    let mut slot: SparseSlot<&str> = SparseSlot::new(5);
    let _iter_mut = slot.iter_mut();
    // This would not compile: let _also_slot = &slot;
}

#[test]
fn iterator_order() {
    let mut slot = SparseSlot::new(5);

    let id0 = Id::new(0, 0);
    let id2 = Id::new(2, 0);
    let id4 = Id::new(4, 0);

    slot.try_set(id2, "second").unwrap();
    slot.try_set(id0, "first").unwrap();
    slot.try_set(id4, "third").unwrap();

    let items: Vec<_> = slot.iter().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].1, &"first");
    assert_eq!(items[1].1, &"second");
    assert_eq!(items[2].1, &"third");

    for (_, value) in slot.iter_mut() {
        *value = "changed";
    }
    assert_eq!(slot.get(id0), Some(&"changed"));
}

#[test]
fn iterator_modifications() {
    let mut slot = SparseSlot::new(5);

    let id0 = Id::new(0, 0);
    let id2 = Id::new(2, 0);
    let id4 = Id::new(4, 0);

    slot.try_set(id0, "first").unwrap();
    slot.try_set(id2, "second").unwrap();
    slot.try_set(id4, "third").unwrap();

    slot.remove(id2);
    let items: Vec<_> = slot.iter().collect();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].1, &"first");
    assert_eq!(items[1].1, &"third");

    slot.remove(id0);
    let items: Vec<_> = slot.iter().collect();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].1, &"third");
}

#[test]

fn specialized_iterators() {
    let mut slot = SparseSlot::new(3);

    let id0 = Id::new(0, 0);
    let id1 = Id::new(1, 0);

    slot.try_set(id0, "first").unwrap();
    slot.try_set(id1, "second").unwrap();

    let keys: Vec<_> = slot.keys().collect();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], id0);
    assert_eq!(keys[1], id1);

    let values: Vec<_> = slot.values().collect();
    assert_eq!(values.len(), 2);
    assert_eq!(values, vec![&"first", &"second"]);

    for value in slot.values_mut() {
        *value = "changed";
    }
    assert_eq!(slot.get(id0), Some(&"changed"));
    assert_eq!(slot.get(id1), Some(&"changed"));
}

#[test]
fn drain() {
    let mut slot = SparseSlot::new(3);

    slot.try_set(Id::new(0, 0), "first").unwrap();
    slot.try_set(Id::new(1, 0), "second").unwrap();

    let drained: Vec<_> = slot.drain().collect();
    assert_eq!(drained.len(), 2);
    assert!(slot.is_empty());

    assert!(slot.try_set(Id::new(0, 0), "new").is_err());
    assert!(slot.try_set(Id::new(0, 1), "new").is_ok());
}

#[test]
fn collect_into_slot() {
    let items = vec![(Id::new(0, 0), "first"), (Id::new(1, 0), "second")];

    let slot: SparseSlot<&str> = items.into_iter().collect();
    assert_eq!(slot.len(), 2);
    assert_eq!(slot.get(Id::new(0, 0)), Some(&"first"));
    assert_eq!(slot.get(Id::new(1, 0)), Some(&"second"));
}

#[test]
fn first_id() {
    let mut slot = SparseSlot::new(5);
    assert_eq!(slot.first_id(), None);

    let id2 = Id::new(2, 0);
    slot.try_set(id2, "second").unwrap();
    assert_eq!(slot.first_id(), Some(id2));

    let id0 = Id::new(0, 0);
    slot.try_set(id0, "first").unwrap();
    assert_eq!(slot.first_id(), Some(id0));

    slot.remove(id0);
    assert_eq!(slot.first_id(), Some(id2));
}
