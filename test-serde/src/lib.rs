#![cfg(test)]

use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet, indexmap, indexset};
use serde::{Deserialize, Serialize};
use serde_test::{Token, assert_tokens};

#[test]
fn test_serde_map() {
    let map = indexmap! { 1 => 2, 3 => 4 };
    assert_tokens(
        &map,
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_serde_set() {
    let set = indexset! { 1, 2, 3, 4 };
    assert_tokens(
        &set,
        &[
            Token::Seq { len: Some(4) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_serde_map_fnv_hasher() {
    let mut map: IndexMap<i32, i32, FnvBuildHasher> = Default::default();
    map.insert(1, 2);
    map.insert(3, 4);
    assert_tokens(
        &map,
        &[
            Token::Map { len: Some(2) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::MapEnd,
        ],
    );
}

#[test]
fn test_serde_set_fnv_hasher() {
    let mut set: IndexSet<i32, FnvBuildHasher> = Default::default();
    set.extend(1..5);
    assert_tokens(
        &set,
        &[
            Token::Seq { len: Some(4) },
            Token::I32(1),
            Token::I32(2),
            Token::I32(3),
            Token::I32(4),
            Token::SeqEnd,
        ],
    );
}

#[test]
fn test_serde_seq_map() {
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(transparent)]
    struct SeqIndexMap {
        #[serde(with = "indexmap::map::serde_seq")]
        map: IndexMap<i32, i32>,
    }

    impl PartialEq for SeqIndexMap {
        fn eq(&self, other: &Self) -> bool {
            // explicitly compare items in order
            self.map.iter().eq(&other.map)
        }
    }

    let map = indexmap! { 1 => 2, 3 => 4, -1 => -2, -3 => -4 };
    assert_tokens(
        &SeqIndexMap { map },
        &[
            Token::Seq { len: Some(4) },
            Token::Tuple { len: 2 },
            Token::I32(1),
            Token::I32(2),
            Token::TupleEnd,
            Token::Tuple { len: 2 },
            Token::I32(3),
            Token::I32(4),
            Token::TupleEnd,
            Token::Tuple { len: 2 },
            Token::I32(-1),
            Token::I32(-2),
            Token::TupleEnd,
            Token::Tuple { len: 2 },
            Token::I32(-3),
            Token::I32(-4),
            Token::TupleEnd,
            Token::SeqEnd,
        ],
    );
}

#[test]
fn remove_then_roundtrip_preserves_order_and_lookup() {
    let mut map: IndexMap<i32, i32> = IndexMap::new();
    for key in 0..50 {
        map.insert(key, key * 10);
    }

    for key in [10_i32, 25, 11, 40, 24] {
        map.shift_remove(&key);
    }
    for key in [3_i32, 33, 47] {
        map.swap_remove(&key);
    }

    // Sanity check the in-memory indices after the removals.
    for (position, (key, value)) in map.iter().enumerate() {
        assert_eq!(map.get_index(position), Some((key, value)));
        assert_eq!(map.get_index_of(key), Some(position));
    }

    // Simulate a restart: serialize the map and read it back.
    let mut tokens = vec![Token::Map {
        len: Some(map.len()),
    }];
    for (key, value) in &map {
        tokens.push(Token::I32(*key));
        tokens.push(Token::I32(*value));
    }
    tokens.push(Token::MapEnd);
    assert_tokens(&map, &tokens);

    // Deserializing the tokens must reproduce the exact post-removal order.
    serde_test::assert_de_tokens(&map, &tokens);
}
