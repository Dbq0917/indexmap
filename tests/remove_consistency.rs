use indexmap::IndexMap;

fn assert_in_sync(map: &IndexMap<i32, i32>, reference: &[(i32, i32)]) {
    assert_eq!(map.len(), reference.len());

    for (position, &(key, value)) in reference.iter().enumerate() {
        // Lookup by position must return the entry at that position.
        assert_eq!(map.get_index(position), Some((&key, &value)));
        // Lookup by key must report the same position.
        assert_eq!(map.get_full(&key), Some((position, &key, &value)));
        assert_eq!(map.get_index_of(&key), Some(position));
    }

    // Iteration order must match the dense entry array.
    let collected: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(collected, reference);
}

fn deterministic_seed(step: usize) -> u64 {
    // Simple LCG so the test stays reproducible without extra dependencies.
    let mut state = 0x1234_5678_9abc_def0_u64 ^ step as u64;
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    state >> 33
}

#[test]
fn shift_remove_middle_keeps_indices_in_sync() {
    let mut map = IndexMap::new();
    let mut reference: Vec<(i32, i32)> = Vec::new();
    for key in 0..50 {
        map.insert(key, key * 10);
        reference.push((key, key * 10));
    }

    // Remove several middle entries, interleaved with position lookups.
    for key in [10_i32, 25, 11, 40, 24, 0, 49] {
        let position = map.get_index_of(&key).unwrap();
        assert_eq!(map.shift_remove_full(&key), Some((position, key, key * 10)));
        reference.remove(position);
        assert_in_sync(&map, &reference);
    }
}

#[test]
fn swap_remove_middle_keeps_indices_in_sync() {
    let mut map = IndexMap::new();
    let mut reference: Vec<(i32, i32)> = Vec::new();
    for key in 0..50 {
        map.insert(key, key * 10);
        reference.push((key, key * 10));
    }

    for key in [10_i32, 25, 11, 40, 24, 0, 49] {
        let position = map.get_index_of(&key).unwrap();
        assert_eq!(map.swap_remove_full(&key), Some((position, key, key * 10)));
        reference.swap_remove(position);
        assert_in_sync(&map, &reference);
    }
}

#[test]
fn mixed_removals_then_reinsert() {
    let mut map: IndexMap<i32, i32> = IndexMap::new();
    for key in 0..100 {
        map.insert(key, key);
    }

    for key in 0..100 {
        if key % 3 == 0 {
            map.shift_remove(&key);
        } else if key % 3 == 1 {
            map.swap_remove(&key);
        }
    }

    // Reinsert some of the removed keys; indices must stay coherent.
    for key in (0..100).step_by(2) {
        map.insert(key, key + 1000);
    }

    let mut expected: Vec<i32> = (0..100)
        .filter(|&key| key % 3 == 2 || key % 2 == 0)
        .collect();
    expected.sort_unstable();

    let mut present: Vec<i32> = Vec::new();
    for (position, (key, value)) in map.iter().enumerate() {
        assert_eq!(map.get_index(position), Some((key, value)));
        assert_eq!(map.get_index_of(key), Some(position));
        present.push(*key);
    }
    present.sort_unstable();
    assert_eq!(present, expected);
}

#[test]
fn fuzz_shift_and_swap_removals() {
    // Exercise both heuristics in `decrement_indices` across many sizes,
    // including cases where removed ranges straddle half the table capacity.
    for round in 0..64 {
        let capacity_buckets = 2 + round * 2;
        let mut map: IndexMap<i32, i32> = IndexMap::with_capacity(capacity_buckets);
        let mut reference: Vec<(i32, i32)> = Vec::new();
        for key in 0..capacity_buckets as i32 {
            map.insert(key, key);
            reference.push((key, key));
        }

        let mut step = 0;
        while map.len() > 1 {
            let position = deterministic_seed(round * 1000 + step) as usize % map.len();
            let (key, _) = map.get_index(position).unwrap();
            let key = *key;
            if deterministic_seed(round * 1000 + step + 1) % 2 == 0 {
                assert_eq!(map.shift_remove_full(&key), Some((position, key, key)));
                reference.remove(position);
            } else {
                assert_eq!(map.swap_remove_full(&key), Some((position, key, key)));
                reference.swap_remove(position);
            }
            assert_in_sync(&map, &reference);
            step += 1;
        }
    }
}
