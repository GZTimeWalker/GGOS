use core::num::NonZeroUsize;

use alloc::sync::Arc;
use lru::LruCache;
use spin::{Mutex, RwLock};
use storage::*;

pub type ATABlockCache = BlockCache<Block512>;
pub type ATACachedDevice = CachedDevice<Block512, LruCacheImpl>;
pub type LruValue = Arc<RwLock<ATABlockCache>>;
pub type LruSharedInner = Arc<Mutex<LruCache<usize, LruValue>>>;

pub struct LruCacheImpl {
    inner: LruSharedInner,
}

impl LruCacheImpl {
    pub fn new() -> Self {
        let size = NonZeroUsize::new(256).unwrap();
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(size))),
        }
    }

    pub fn inner(&self) -> LruSharedInner {
        self.inner.clone()
    }
}

impl Default for LruCacheImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheManager<Block512> for LruCacheImpl {
    fn get(&self, key: &usize) -> Option<LruValue> {
        let mut inner = self.inner.lock();
        inner.get(key).cloned()
    }

    fn put(&self, key: usize, value: BlockCache<Block512>) {
        let mut inner = self.inner.lock();
        inner.put(key, Arc::new(RwLock::new(value)));
    }
}
