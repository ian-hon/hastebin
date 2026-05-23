use engine::models::Paste;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

// LRU cache
// once the cache is full, we replace the least recently used record
// side note: why is lru called lru and not mru
// this world is confusing
#[derive(Clone)]
pub struct PasteCache {
    cache: Arc<Mutex<CacheInner>>,
    max_size: usize,
}

struct CacheInner {
    map: HashMap<i64, Paste>,
    order: VecDeque<i64>,
}

impl PasteCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(CacheInner {
                map: HashMap::new(),
                order: VecDeque::new(),
            })),
            max_size,
        }
    }

    // CRUD
    // no update, only CRD

    pub fn get(&self, id: i64) -> Option<Paste> {
        self.cache.lock().unwrap().map.get(&id).cloned()
    }

    pub fn insert(&self, id: i64, paste: Paste) {
        let mut cache = self.cache.lock().unwrap();

        // if it already exists, remove it
        // trust me itll get added back later
        if cache.map.contains_key(&id) {
            cache.order.retain(|&existing_id| existing_id != id);
        }

        cache.map.insert(id, paste);
        cache.order.push_back(id); // see, whatd i say

        // lru logic here
        while cache.order.len() > self.max_size {
            // honestly it doesnt have to be a loop if everythings done right
            // but lets just loop in case
            if let Some(oldest_id) = cache.order.pop_front() {
                cache.map.remove(&oldest_id);
            }
        }
    }

    pub fn remove(&self, id: i64) {
        // need to remove from the map and the order vecdeque
        let mut cache = self.cache.lock().unwrap();
        cache.map.remove(&id);
        cache.order.retain(|&existing_id| existing_id != id);
    }

    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_paste(id: i64) -> Paste {
        Paste {
            id,
            content: format!("content for paste {id}"),
            title: None,
            author: None,
            checksum_passphrase: None,
            views: 0,
            comments_enabled: true,
            created_at: 0,
            expires_at: None,
            forked_from: None,
        }
    }

    #[test]
    fn insert_and_get_returns_paste() {
        let cache = PasteCache::new(10);
        let paste = make_paste(1);
        cache.insert(1, paste.clone());
        let result = cache.get(1).expect("expected paste in cache");
        assert_eq!(result.id, 1);
        assert_eq!(result.content, paste.content);
    }

    #[test]
    fn get_missing_key_returns_none() {
        let cache = PasteCache::new(10);
        assert!(cache.get(999).is_none());
    }

    #[test]
    fn remove_deletes_entry() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));
        cache.remove(1);
        assert!(cache.get(1).is_none());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn remove_nonexistent_is_a_no_op() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));
        cache.remove(99); // should not panic or remove entry 1
        assert!(cache.get(1).is_some());
    }

    #[test]
    fn len_tracks_insertions_and_removals() {
        let cache = PasteCache::new(10);
        assert_eq!(cache.len(), 0);
        cache.insert(1, make_paste(1));
        assert_eq!(cache.len(), 1);
        cache.insert(2, make_paste(2));
        assert_eq!(cache.len(), 2);
        cache.remove(1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn lru_evicts_oldest_entry_when_full() {
        let cache = PasteCache::new(3);
        cache.insert(1, make_paste(1));
        cache.insert(2, make_paste(2));
        cache.insert(3, make_paste(3));
        // inserting a 4th entry should evict entry 1 (LRU)
        cache.insert(4, make_paste(4));
        assert!(cache.get(1).is_none(), "entry 1 should have been evicted");
        assert!(cache.get(2).is_some());
        assert!(cache.get(3).is_some());
        assert!(cache.get(4).is_some());
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn reinserting_existing_key_moves_it_to_most_recent() {
        let cache = PasteCache::new(3);
        cache.insert(1, make_paste(1));
        cache.insert(2, make_paste(2));
        cache.insert(3, make_paste(3));

        // re-insert 1 — it should now be the most recently used
        cache.insert(1, make_paste(1));

        // inserting a 4th entry should evict entry 2 (now the LRU), not 1
        cache.insert(4, make_paste(4));
        assert!(
            cache.get(1).is_some(),
            "entry 1 should not have been evicted"
        );
        assert!(cache.get(2).is_none(), "entry 2 should have been evicted");
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn cache_with_max_size_zero_stays_empty() {
        let cache = PasteCache::new(0);
        cache.insert(1, make_paste(1));
        assert_eq!(cache.len(), 0);
        assert!(cache.get(1).is_none());
    }

    #[test]
    fn clone_shares_underlying_state() {
        let cache = PasteCache::new(10);
        let clone = cache.clone();
        cache.insert(1, make_paste(1));
        // clone should see the same entry because they share the Arc<Mutex<_>>
        assert!(clone.get(1).is_some());
    }
}
