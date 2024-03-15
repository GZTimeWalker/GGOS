use super::Result;

pub trait Device<T> {
    /// Read data from the device into the buffer
    fn read(&self, buf: &mut [T], offset: usize, size: usize) -> Result<usize>;

    /// Write data from the buffer to the device
    fn write(&mut self, buf: &[T], offset: usize, size: usize) -> Result<usize>;
}

pub trait BlockDevice<B>: Send + Sync + 'static
where
    B: AsMut<[u8]> + AsRef<[u8]> + Default + Send + Sync + 'static,
{
    /// Returns the number of blocks in the device
    fn block_count(&self) -> Result<usize>;

    /// Reads a block from the device into the provided buffer
    fn read_block(&self, offset: usize, block: &mut B) -> Result<()>;

    /// Writes a block to the device from the provided buffer
    fn write_block(&self, offset: usize, block: &B) -> Result<()>;
}
