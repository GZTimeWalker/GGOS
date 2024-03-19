use spin::RwLock;

use super::*;

pub struct BlockCache<B>
where
    B: BlockTrait,
{
    inner: B,
    device: Arc<dyn BlockDevice<B>>,
    offset: usize,
    modified: bool,
}

impl<B: BlockTrait> BlockCache<B> {
    pub fn new(offset: usize, device: Arc<dyn BlockDevice<B>>, inner: B, modified: bool) -> Self {
        Self {
            device,
            offset,
            inner,
            modified,
        }
    }

    #[inline]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    #[inline]
    pub fn save(&mut self, data: &B) -> Result<()> {
        self.inner.as_mut().copy_from_slice(data.as_ref());
        self.modified = true;
        Ok(())
    }

    #[inline]
    pub fn load(&self, data: &mut B) -> Result<()> {
        data.as_mut().copy_from_slice(self.inner.as_ref());
        Ok(())
    }
}

impl<B: BlockTrait> Drop for BlockCache<B> {
    fn drop(&mut self) {
        // This can be implemented as kernel async task
        if self.modified {
            match self.device.write_block(self.offset, &self.inner) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to write block to device: {:?}", e);
                }
            }
        }
    }
}

pub trait CacheManager<B>: Sync + Send + 'static
where
    B: BlockTrait,
{
    /// Get a block from the cache
    fn get(&self, key: &usize) -> Option<Arc<RwLock<BlockCache<B>>>>;

    /// Put a block into the cache
    fn put(&self, key: usize, value: BlockCache<B>);
}

pub struct CachedDevice<B, C>
where
    B: BlockTrait,
    C: CacheManager<B>,
{
    cache: C,
    device: Arc<dyn BlockDevice<B>>,
}

impl<B, C> CachedDevice<B, C>
where
    B: BlockTrait,
    C: CacheManager<B>,
{
    pub fn new(device: impl BlockDevice<B>, cache: C) -> Self {
        Self {
            device: Arc::new(device),
            cache,
        }
    }

    fn save_cache(&self, offset: usize, block: B, modified: bool) {
        let cache = BlockCache::new(offset, self.device.clone(), block, modified);
        self.cache.put(offset, cache);
    }
}

impl<B, C> BlockDevice<B> for CachedDevice<B, C>
where
    B: BlockTrait,
    C: CacheManager<B>,
{
    fn block_count(&self) -> Result<usize> {
        self.device.block_count()
    }

    fn read_block(&self, offset: usize, block: &mut B) -> Result<()> {
        match self.cache.get(&offset) {
            Some(cache) => {
                log::trace!("Cache hit for block {}", offset);
                cache.read().load(block)?;
            }
            None => {
                log::trace!("Cache missed for block {}", offset);
                self.device.read_block(offset, block)?;
                self.save_cache(offset, block.clone(), false);
            }
        };

        Ok(())
    }

    fn write_block(&self, offset: usize, block: &B) -> Result<()> {
        match self.cache.get(&offset) {
            Some(cache) => {
                cache.write().save(block)?;
            }
            None => {
                self.save_cache(offset, block.clone(), true);
            }
        };
        Ok(())
    }
}
