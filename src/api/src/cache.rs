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
