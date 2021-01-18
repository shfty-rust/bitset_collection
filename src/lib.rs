use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    iter::FromIterator,
    marker::PhantomData,
};

use hibitset::{BitIter, BitSet, BitSetLike};

pub use collection_trait;
use collection_trait::Collection;

/// `BitSetCollection` wrapping a `Vec`
pub type BitSetVec<'a, K, V> = BitSetCollection<'a, K, Vec<V>>;
/// `BitSetCollection` wrapping an immutable slice
pub type BitSetSlice<'a, K, V> = BitSetCollection<'a, K, &'a [V]>;
/// `BitSetCollection` wrapping a mutable slice
pub type BitSetMutSlice<'a, K, V> = BitSetCollection<'a, K, &'a mut [V]>;
/// `BitSetCollection` wrapping a `VecDeque`
pub type BitSetVecDeque<'a, K, V> = BitSetCollection<'a, K, std::collections::VecDeque<V>>;
/// `BitSetCollection` wrapping a `BTreeMap`
pub type BitSetBTreeMap<'a, K, V> = BitSetCollection<'a, K, std::collections::BTreeMap<K, V>>;
/// `BitSetCollection` wrapping a `HashMap`
pub type BitSetHashMap<'a, K, V> = BitSetCollection<'a, K, std::collections::HashMap<K, V>>;

/// Wrapper for overriding a `Collection`'s key handling with a `BitSet`.
///
/// Useful for accellerating lookups on map-like types, or to augment list-like types with distinct key tracking.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BitSetCollection<'a, K, C>
where
    C: Collection<'a, K>,
{
    bitset: BitSet,
    collection: C,
    _phantom: PhantomData<&'a K>,
}

impl<'a, C, K> BitSetCollection<'a, K, C>
where
    K: TryInto<u32>,
    <K as TryInto<u32>>::Error: Debug,
    C: for<'b> Collection<'b, K>,
{
    pub fn new(collection: C) -> Self {
        BitSetCollection {
            bitset: collection
                .keys()
                .map(|key| key.try_into().unwrap())
                .collect(),
            collection,
            _phantom: Default::default(),
        }
    }
}

impl<'a, C, K, V> FromIterator<(K, V)> for BitSetCollection<'a, K, C>
where
    K: TryInto<u32>,
    <K as TryInto<u32>>::Error: Debug,
    C: Default + for<'b> Collection<'b, K, Item = V>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut collection: C = Default::default();
        for (key, value) in iter {
            collection.insert(key, value);
        }
        BitSetCollection::new(collection)
    }
}

impl<'a, C, K> Collection<'a, K> for BitSetCollection<'a, K, C>
where
    C: Collection<'a, K>,
    K: Copy + TryInto<u32> + TryFrom<u32>,
    <K as TryInto<u32>>::Error: Debug,
    <K as TryFrom<u32>>::Error: Debug,
{
    type Item = C::Item;

    type KeyIter = std::iter::Map<BitIter<BitSet>, fn(u32) -> K>;

    fn get(&'a self, key: &K) -> Option<&Self::Item> {
        if self.bitset.contains((*key).try_into().unwrap()) {
            Some(self.collection.get_unchecked(key))
        } else {
            None
        }
    }

    fn insert(&mut self, key: K, value: Self::Item) -> Option<Self::Item> {
        self.bitset.add(key.try_into().unwrap());
        self.collection.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<Self::Item> {
        self.bitset.remove((*key).try_into().unwrap());
        self.collection.remove(key)
    }

    fn keys(&'a self) -> Self::KeyIter {
        self.bitset
            .clone()
            .iter()
            .map(|key| key.try_into().unwrap())
    }

    fn contains_key(&'a self, key: &K) -> bool {
        self.bitset.contains((*key).try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitset_vec_insert() {
        let mut collection = BitSetVec::<usize, f32>::default();
        assert!(!collection.contains_key(&2));
        collection.insert(2, 10.0);
        assert!(collection.contains_key(&2));
    }

    #[test]
    fn bitset_vec_remove() {
        let mut collection = vec![(0, 1), (2, 3), (4, 5)]
            .into_iter()
            .collect::<BitSetVec<usize, usize>>();
        assert!(collection.contains_key(&2));
        collection.remove(&2);
        assert!(!collection.contains_key(&2));
    }

    #[test]
    fn bitset_btree_map_insert() {
        let mut collection = BitSetBTreeMap::<usize, f32>::default();
        assert!(!collection.contains_key(&2));
        collection.insert(2, 10.0);
        assert!(collection.contains_key(&2));
    }

    #[test]
    fn bitset_btree_map_remove() {
        let mut collection = vec![(0, 1), (2, 3), (4, 5)]
            .into_iter()
            .collect::<BitSetBTreeMap<usize, usize>>();
        assert!(collection.contains_key(&2));
        collection.remove(&2);
        assert!(!collection.contains_key(&2));
    }
}
