use alloc::{collections::BTreeMap, vec::Vec};
use boot::KernelPages;
use spin::RwLock;
use x86_64::structures::paging::{
    page::{PageRange, PageRangeInclusive},
    Page,
};

use crate::filesystem::StdIO;

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,
    pub(super) file_handles: Arc<RwLock<BTreeMap<u8, Resource>>>,
    pub(super) semaphores: Arc<RwLock<SemaphoreSet>>,

    // process specific data
    pub(super) code_segments: Option<Vec<PageRangeInclusive>>,
    pub(super) stack_segment: Option<PageRange>,
    pub(super) code_memory_usage: usize,
    pub(super) stack_memory_usage: usize,
}

impl Default for ProcessData {
    fn default() -> Self {
        let mut file_handles = BTreeMap::new();

        // stdin, stdout, stderr
        file_handles.insert(0, Resource::Console(StdIO::Stdin));
        file_handles.insert(1, Resource::Console(StdIO::Stdout));
        file_handles.insert(2, Resource::Console(StdIO::Stderr));

        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            semaphores: Arc::new(RwLock::new(SemaphoreSet::default())),
            code_segments: None,
            stack_segment: None,
            file_handles: Arc::new(RwLock::new(file_handles)),
            code_memory_usage: 0,
            stack_memory_usage: 0,
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, res: Resource) -> u8 {
        let fd = self.file_handles.read().len() as u8;
        self.file_handles.write().insert(fd, res);
        fd
    }

    pub fn close(&mut self, fd: u8) -> bool {
        self.file_handles.write().remove(&fd).is_some()
    }

    pub fn handle(&self, fd: u8) -> Option<Resource> {
        self.file_handles.read().get(&fd).cloned()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn memory_usage(&self) -> usize {
        self.code_memory_usage + self.stack_memory_usage
    }

    pub fn set_env(self, key: &str, val: &str) -> Self {
        self.env.write().insert(key.into(), val.into());
        self
    }

    pub fn set_stack(mut self, start: u64, size: u64) -> Self {
        let start = Page::containing_address(VirtAddr::new(start));
        self.stack_segment = Some(Page::range(start, start + size));
        self.stack_memory_usage = size as usize;
        self
    }

    pub fn set_kernel_code(mut self, pages: &KernelPages) -> Self {
        let mut size = 0;
        let owned_pages = pages
            .iter()
            .map(|page| {
                size += page.count();
                *page
            })
            .collect();
        self.code_segments = Some(owned_pages);
        self.code_memory_usage = size;
        self
    }

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        if let Some(stack_range) = self.stack_segment {
            let addr = addr.as_u64();
            let cur_stack_bot = stack_range.start.start_address().as_u64();
            trace!("Current stack bot: {:#x}", cur_stack_bot);
            trace!("Address to access: {:#x}", addr);
            addr & STACK_START_MASK == cur_stack_bot & STACK_START_MASK
        } else {
            false
        }
    }

    #[inline]
    pub fn new_sem(&mut self, key: u32, value: usize) -> bool {
        self.semaphores.write().insert(key, value)
    }

    #[inline]
    pub fn remove_sem(&mut self, key: u32) -> bool {
        self.semaphores.write().remove(key)
    }

    #[inline]
    pub fn sem_up(&mut self, key: u32) -> SemaphoreResult {
        self.semaphores.read().up(key)
    }

    #[inline]
    pub fn sem_down(&mut self, key: u32, pid: ProcessId) -> SemaphoreResult {
        self.semaphores.read().down(key, pid)
    }
}
