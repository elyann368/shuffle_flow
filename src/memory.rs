//! Explicit retention limits for an app that also needs to run on 8 GB Macs.
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

pub const IMAGE_BUDGET: usize = 16 * 1024 * 1024;
pub const INDEX_BUDGET: usize = 48 * 1024 * 1024;
pub const PREVIEW_CONCURRENCY: usize = 2;

/// A weighted LRU with both byte and entry limits. Failed previews also count
/// toward the entry limit, so visiting unsupported files cannot grow it forever.
pub struct Cache<K, V> {
    entries: HashMap<K, (V, usize, u64)>,
    max_entries: usize,
    budget: usize,
    bytes: usize,
    clock: u64,
}
impl<K: Eq + Hash + Clone, V> Cache<K, V> {
    pub fn new(max_entries: usize, budget: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            budget,
            bytes: 0,
            clock: 0,
        }
    }
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.clock += 1;
        let entry = self.entries.get_mut(key)?;
        entry.2 = self.clock;
        Some(&entry.0)
    }
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let (value, cost, _) = self.entries.remove(key)?;
        self.bytes -= cost;
        Some(value)
    }
    pub fn remove_matching(&mut self, matches: impl Fn(&K) -> bool) -> Vec<V> {
        let keys: Vec<_> = self
            .entries
            .keys()
            .filter(|key| matches(key))
            .cloned()
            .collect();
        keys.iter().filter_map(|key| self.remove(key)).collect()
    }
    pub fn set_budget(&mut self, budget: usize) -> Vec<V> {
        self.budget = budget;
        let mut removed = Vec::new();
        while self.bytes > budget {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.2)
                .map(|(k, _)| k.clone());
            let Some(oldest) = oldest else { break };
            if let Some(value) = self.remove(&oldest) {
                removed.push(value);
            }
        }
        removed
    }
    /// Return every removed value, allowing the caller to release GPU textures
    /// as well as CPU buffers. Never silently retain oversized values.
    pub fn insert(&mut self, key: K, value: V, cost: usize) -> Vec<V> {
        let mut removed = Vec::new();
        if let Some(old) = self.remove(&key) {
            removed.push(old);
        }
        if cost > self.budget || self.max_entries == 0 {
            removed.push(value);
            return removed;
        }
        while self.entries.len() >= self.max_entries || self.bytes + cost > self.budget {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.2)
                .map(|(k, _)| k.clone());
            let Some(oldest) = oldest else { break };
            if let Some(old) = self.remove(&oldest) {
                removed.push(old);
            }
        }
        self.clock += 1;
        self.bytes += cost;
        self.entries.insert(key, (value, cost, self.clock));
        removed
    }
}

/// Keep only the best matches while scoring instead of collecting/sorting
/// every match in a large home-directory index.
pub struct TopK<T: Ord> {
    limit: usize,
    heap: BinaryHeap<Reverse<T>>,
}
impl<T: Ord> TopK<T> {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            heap: BinaryHeap::new(),
        }
    }
    pub fn push(&mut self, value: T) {
        if self.limit == 0 {
            return;
        }
        if self.heap.len() < self.limit {
            self.heap.push(Reverse(value));
        } else if self.heap.peek().is_some_and(|min| value > min.0) {
            self.heap.pop();
            self.heap.push(Reverse(value));
        }
    }
    pub fn merge(mut self, other: Self) -> Self {
        for Reverse(value) in other.heap {
            self.push(value);
        }
        self
    }
    pub fn into_sorted(self) -> Vec<T> {
        let mut values: Vec<_> = self.heap.into_iter().map(|Reverse(v)| v).collect();
        values.sort_unstable_by(|a, b| b.cmp(a));
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_retention_obeys_bytes_and_keeps_recently_viewed_items() {
        let mut cache = Cache::new(4, 10);
        cache.insert("a", 1, 4);
        cache.insert("b", 2, 4);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.insert("c", 3, 4), vec![2]);
        assert_eq!(cache.bytes, 8);
        assert!(cache.get(&"b").is_none());
        assert_eq!(cache.insert("a", 4, 2), vec![1]);
        assert_eq!(cache.bytes, 6);
        assert_eq!(cache.remove(&"a"), Some(4));
        assert_eq!(cache.bytes, 4);
    }
    #[test]
    fn failed_previews_are_bounded_and_oversized_images_are_released() {
        let mut cache = Cache::new(2, 10);
        for key in 0..10_000 {
            cache.insert(key, None::<usize>, 0);
        }
        assert_eq!(cache.entries.len(), 2);
        assert_eq!(cache.insert(10001, Some(99), 11), vec![Some(99)]);
        assert_eq!(cache.entries.len(), 2);
    }
    #[test]
    fn reducing_budget_releases_images_immediately() {
        let mut cache = Cache::new(10, 100);
        cache.insert("a", 1, 30);
        cache.insert("b", 2, 30);
        assert_eq!(cache.set_budget(32), vec![1]);
        assert_eq!(cache.bytes, 30);
        assert_eq!(cache.set_budget(0), vec![2]);
        assert_eq!(cache.bytes, 0);
    }
    #[test]
    fn bounded_parallel_ranking_matches_full_sort() {
        let values: Vec<_> = (0..30_000)
            .map(|i| ((i * 17) % 1003, Reverse(i % 73), Reverse(i)))
            .collect();
        let mut left = TopK::new(40);
        let mut right = TopK::new(40);
        for (i, value) in values.iter().copied().enumerate() {
            if i % 2 == 0 {
                left.push(value);
            } else {
                right.push(value);
            }
        }
        let mut expected = values;
        expected.sort_unstable_by(|a, b| b.cmp(a));
        expected.truncate(40);
        assert_eq!(left.merge(right).into_sorted(), expected);
        let mut empty = TopK::new(0);
        empty.push(7);
        assert!(empty.into_sorted().is_empty());
    }
}
