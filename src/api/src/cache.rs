use engine::models::Paste;
use sqlx::{Pool, Postgres};
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
    // note: views can be larger than map. all that matters is we keep track of the views
    views: HashMap<i64, i64>, // views will be synchronised with db every 5 mins
}

impl PasteCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(CacheInner {
                map: HashMap::new(),
                order: VecDeque::new(),
                views: HashMap::new(),
            })),
            max_size,
        }
    }

    // CRUD
    // no update, only CRD

    pub fn get(&self, id: i64) -> Option<Paste> {
        let mut lock = self.cache.lock().unwrap();
        let updated_views = {
            let paste = lock.map.get_mut(&id)?;
            paste.views += 1;
            paste.views
        };

        // update order
        lock.order.retain(|&existing_id| existing_id != id);
        lock.order.push_back(id);

        // update the views hashmap
        lock.views.insert(id, updated_views);

        lock.map.get(&id).cloned()
    }

    pub fn insert(&self, id: i64, mut paste: Paste) {
        let mut cache = self.cache.lock().unwrap();

        // if it already exists, remove it
        // trust me itll get added back later
        if cache.map.contains_key(&id) {
            cache.order.retain(|&existing_id| existing_id != id);
        }

        // if the paste gets removed then re-inserted, we use the views hashmap
        // its the only thing thatll persist longer
        if let Some(&view_count) = cache.views.get(&id) {
            paste.views = view_count;
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

    pub async fn synchronise(&self, pool: &Pool<Postgres>) {
        let views = {
            let mut lock = self.cache.lock().unwrap();
            std::mem::take(&mut lock.views)
        };

        // then set the views
        for (id, view_count) in views {
            println!("setting {id} to {view_count}");
            Paste::set_views(id, view_count, pool).await;
        }
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

    #[test]
    fn get_increments_view_count() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));

        let result1 = cache.get(1).unwrap();
        assert_eq!(result1.views, 1);

        let result2 = cache.get(1).unwrap();
        assert_eq!(result2.views, 2);

        let result3 = cache.get(1).unwrap();
        assert_eq!(result3.views, 3);
    }

    #[test]
    fn get_preserves_initial_view_count() {
        let cache = PasteCache::new(10);
        let mut paste = make_paste(1);
        paste.views = 100;
        cache.insert(1, paste);

        let result = cache.get(1).unwrap();
        assert_eq!(result.views, 101, "should increment from initial views");
    }

    #[test]
    fn synchronise_drains_views_map() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));
        cache.insert(2, make_paste(2));

        cache.get(1);
        cache.get(1);
        cache.get(2);

        let views = {
            let lock = cache.cache.lock().unwrap();
            lock.views.clone()
        };

        assert_eq!(views.get(&1), Some(&2));
        assert_eq!(views.get(&2), Some(&1));
    }

    #[test]
    fn get_updates_lru_order_preventing_eviction() {
        let cache = PasteCache::new(3);
        cache.insert(1, make_paste(1));
        cache.insert(2, make_paste(2));
        cache.insert(3, make_paste(3));

        // access entry 1 to make it most recent
        cache.get(1);

        // inserting entry 4 should evict entry 2 (LRU), not entry 1
        cache.insert(4, make_paste(4));

        assert!(cache.get(1).is_some(), "entry 1 should still exist");
        assert!(cache.get(2).is_none(), "entry 2 should have been evicted");
        assert!(cache.get(3).is_some(), "entry 3 should still exist");
        assert!(cache.get(4).is_some(), "entry 4 should exist");
    }

    #[test]
    fn view_tracking_persists_after_cache_eviction() {
        let cache = PasteCache::new(2);
        cache.insert(1, make_paste(1));
        cache.insert(2, make_paste(2));

        // increment views for entry 1
        cache.get(1);
        cache.get(1);

        // evict entry 1 by inserting 3 and 4
        cache.insert(3, make_paste(3));
        cache.insert(4, make_paste(4));

        assert!(
            cache.get(1).is_none(),
            "entry 1 should be evicted from cache"
        );

        // views map should still track entry 1's views even though it's not in the cache
        let views = {
            let lock = cache.cache.lock().unwrap();
            lock.views.clone()
        };

        assert_eq!(
            views.get(&1),
            Some(&2),
            "view count should persist for evicted entries"
        );
    }

    #[test]
    fn multiple_pastes_track_views_independently() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));
        cache.insert(2, make_paste(2));
        cache.insert(3, make_paste(3));

        cache.get(1);
        cache.get(2);
        cache.get(2);
        cache.get(3);
        cache.get(3);
        cache.get(3);

        let p1 = cache.get(1).unwrap();
        let p2 = cache.get(2).unwrap();
        let p3 = cache.get(3).unwrap();

        assert_eq!(p1.views, 2);
        assert_eq!(p2.views, 3);
        assert_eq!(p3.views, 4);
    }

    #[test]
    fn insert_does_not_reset_view_count() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));

        cache.get(1);
        cache.get(1);

        // re-insert the same paste
        cache.insert(1, make_paste(1));

        // view count should be preserved in the views map
        let result = cache.get(1).unwrap();
        assert_eq!(
            result.views, 3,
            "views should continue from where they left off"
        );
    }

    #[test]
    fn remove_keeps_view_tracking() {
        let cache = PasteCache::new(10);
        cache.insert(1, make_paste(1));

        cache.get(1);
        cache.get(1);

        cache.remove(1);

        // views should still be tracked even after removal
        let views = {
            let lock = cache.cache.lock().unwrap();
            lock.views.clone()
        };

        assert_eq!(
            views.get(&1),
            Some(&2),
            "view tracking should persist after removal"
        );
    }
}
