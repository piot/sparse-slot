use sparse_slot::prelude::*;

#[test]
fn test_basic_operations() {
    let mut slot = SparseSlot::new(3);
    let id = Id::new(1, 0);

    assert!(slot.try_set(id, "hello").is_ok());
    assert_eq!(slot.get(id), Some(&"hello"));
    assert_eq!(slot.remove(id), Some("hello"));
    assert_eq!(slot.get(id), None);
    assert_eq!(slot.len(), 0);
}

#[test]
fn test_generation_handling() {
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
fn test_error_conditions() {
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
fn test_iteration() {
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
fn test_clear_and_capacity() {
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
