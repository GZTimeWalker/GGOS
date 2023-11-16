use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure,
    user::{USER_ALLOCATOR, USER_HEAP_SIZE},
    PAGE_SIZE,
};
use crate::utils::Registers;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::paging::PhysFrame;
use x86_64::VirtAddr;
use xmas_elf::ElfFile;

once_mutex!(pub PROCESS_MANAGER: ProcessManager);
guard_access_fn! {
    pub get_process_manager(PROCESS_MANAGER: ProcessManager)
}

pub struct ProcessManager {
    /// pid of the current running process
    cur_pid: ProcessId,
    processes: Vec<Process>,
    exit_code: BTreeMap<ProcessId, isize>,
}

impl ProcessManager {
    pub fn new(init: Process) -> Self {
        let mut processes = Vec::<Process>::new();
        let exit_code = BTreeMap::new();
        let cur_pid = init.pid();
        processes.push(init);
        Self {
            cur_pid,
            processes,
            exit_code,
        }
    }

    fn current_mut(&mut self) -> &mut Process {
        self.processes
            .iter_mut()
            .find(|x| x.pid() == self.cur_pid)
            .unwrap()
    }

    fn pid_mut(&mut self, pid: ProcessId) -> &mut Process {
        self.processes.iter_mut().find(|x| x.pid() == pid).unwrap()
    }

    pub fn current(&self) -> &Process {
        self.processes
            .iter()
            .find(|x| x.pid() == self.cur_pid)
            .unwrap()
    }

    pub fn current_pid(&self) -> ProcessId {
        self.cur_pid
    }

    pub fn add_child(&mut self, child: ProcessId) {
        self.current_mut().add_child(child);
    }

    pub fn save_current(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        let current = self.current_mut();
        if current.is_running() {
            current.tick();
            current.save(regs, sf);
        }
        // trace!("Paused process #{}", self.cur_pid);
    }

    fn get_next_pos(&self) -> usize {
        let cur_pos = self
            .processes
            .iter()
            .position(|x| x.pid() == self.cur_pid)
            .unwrap_or(0);

        let mut next_pos = (cur_pos + 1) % self.processes.len();

        while self.processes[next_pos].status() != ProgramStatus::Ready {
            next_pos = (next_pos + 1) % self.processes.len();
        }

        next_pos
    }

    pub fn wait_pid(&mut self, pid: ProcessId) -> isize {
        if self.exit_code.contains_key(&pid) {
            *self.exit_code.get(&pid).unwrap()
        } else {
            -1
        }
    }

    pub fn still_alive(&self, pid: ProcessId) -> bool {
        self.processes.iter().any(|x| x.pid() == pid)
    }

    pub fn switch_next(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        let pos = self.get_next_pos();
        let p = &mut self.processes[pos];

        // trace!("Next process {} #{}", p.name(), p.pid());
        if p.pid() == self.cur_pid {
            // the next process to be resumed is the same as the current one
            p.resume();
        } else {
            // switch to next process
            p.restore(regs, sf);
            self.cur_pid = p.pid();
        }
    }

    fn get_kernel_page_table(&self) -> PhysFrame {
        let proc = self.processes.first().unwrap();
        proc.page_table_addr()
    }

    pub fn open(&mut self, path: &str, mode: u8) -> Option<u8> {
        let res = match path {
            "/dev/random" => Resource::Random(fs::Random::new(
                crate::utils::clock::now().timestamp() as u64,
            )),
            path => {
                let file = crate::filesystem::try_get_file(path, fs::Mode::try_from(mode).unwrap());

                if file.is_err() {
                    return None;
                }

                Resource::File(file.unwrap())
            }
        };

        trace!("Opening {}...\n{:#?}", path, &res);

        let fd = self.current_mut().open(res);

        Some(fd)
    }

    pub fn close(&mut self, fd: u8) -> bool {
        if fd < 3 {
            false // stdin, stdout, stderr are reserved
        } else {
            self.current_mut().close(fd)
        }
    }

    pub fn spawn(
        &mut self,
        elf: &ElfFile,
        name: String,
        parent: ProcessId,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let mut p = Process::new(
            &mut crate::memory::get_frame_alloc_for_sure(),
            name,
            parent,
            self.get_kernel_page_table(),
            proc_data,
        );
        p.pause();
        p.init_elf(elf);
        p.init_stack_frame(
            VirtAddr::new_truncate(elf.header.pt2.entry_point()),
            VirtAddr::new_truncate(STACK_INIT_TOP),
        );
        trace!("New {:#?}", &p);
        let pid = p.pid();
        self.processes.push(p);
        pid
    }

    // DEPRECATED: do not spawn kernel thread
    // pub fn spawn_kernel_thread(
    //     &mut self,
    //     entry: VirtAddr,
    //     stack_top: VirtAddr,
    //     name: String,
    //     parent: ProcessId,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let mut p = Process::new(
    //         &mut crate::memory::get_frame_alloc_for_sure(),
    //         name,
    //         parent,
    //         self.get_kernel_page_table(),
    //         proc_data,
    //     );
    //     p.pause();
    //     p.init_stack_frame(entry, stack_top);
    //     info!("Spawn process: {}#{}", p.name(), p.pid());
    //     let pid = p.pid();
    //     self.processes.push(p);
    //     pid
    // }

    pub fn print_process_list(&self) {
        let mut output =
            String::from("  PID | PPID | Process Name |  Ticks  |   Memory  | Status\n");
        for p in self.processes.iter() {
            output += format!("{}\n", p).as_str();
        }

        let heap_used = ALLOCATOR.lock().used();
        let heap_size = HEAP_SIZE;

        let user_heap_used = USER_ALLOCATOR.lock().used();
        let user_heap_size = USER_HEAP_SIZE;

        let alloc = get_frame_alloc_for_sure();
        let frames_used = alloc.frames_used();
        let frames_recycled = alloc.recycled_count();
        let frames_total = alloc.frames_total();

        let (sys_used, sys_used_unit) = memory::humanized_size(heap_used as u64);
        let (sys_size, sys_size_unit) = memory::humanized_size(heap_size as u64);

        output += format!(
            "Kernel : {:>6.*} {} / {:>6.*} {} ({:>5.2}%)\n",
            2,
            sys_used,
            sys_used_unit,
            2,
            sys_size,
            sys_size_unit,
            heap_used as f64 / heap_size as f64 * 100.0
        )
        .as_str();

        let (user_used, user_used_unit) = memory::humanized_size(user_heap_used as u64);
        let (user_size, user_size_unit) = memory::humanized_size(user_heap_size as u64);

        output += format!(
            "User   : {:>6.*} {} / {:>6.*} {} ({:>5.2}%)\n",
            2,
            user_used,
            user_used_unit,
            2,
            user_size,
            user_size_unit,
            user_heap_used as f64 / user_heap_size as f64 * 100.0
        )
        .as_str();

        // put used/total frames in MiB
        let (used_size, used_unit) =
            memory::humanized_size((frames_used - frames_recycled) as u64 * PAGE_SIZE);
        let (tot_size, tot_unit) = memory::humanized_size(frames_total as u64 * PAGE_SIZE);

        output += format!(
            "Memory : {:>6.*} {} / {:>6.*} {} ({:>5.2}%) [{} recycled]\n",
            2,
            used_size,
            used_unit,
            2,
            tot_size,
            tot_unit,
            (frames_used - frames_recycled) as f64 / frames_total as f64 * 100.0,
            frames_recycled
        )
        .as_str();

        print!("{}", output);
    }

    pub fn fork(&mut self) {
        let mut p = self.current_mut().fork();
        p.pause();
        self.processes.push(p);
    }

    pub fn kill_self(&mut self, ret: isize) {
        self.kill(self.cur_pid, ret);
    }

    pub fn unblock(&mut self, pid: ProcessId) {
        self.processes.iter_mut().for_each(|p| {
            if p.pid() == pid {
                p.pause()
            }
        });
    }

    pub fn block(&mut self, pid: ProcessId) {
        self.processes.iter_mut().for_each(|p| {
            if p.pid() == pid {
                p.block()
            }
        });
    }

    pub fn handle_page_fault(
        &mut self,
        addr: VirtAddr,
        err_code: PageFaultErrorCode,
    ) -> Result<(), ()> {
        if !err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            let cur_proc = self.current_mut();
            trace!(
                "Page Fault! Checking if {:#x} is on current process's stack",
                addr
            );
            if cur_proc.is_on_stack(addr) {
                cur_proc.try_alloc_new_stack_page(addr).unwrap();
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn kill(&mut self, pid: ProcessId, ret: isize) {
        let p = self.processes.iter().find(|x| x.pid() == pid);

        if p.is_none() {
            warn!("Process #{} not found", pid);
            return;
        }

        let p = p.unwrap();

        debug!(
            "Killing process {}#{} with ret code: {}",
            p.name(),
            pid,
            ret
        );

        trace!("Kill {:#?}", &p);

        let parent = p.parent();
        let children = p.children();
        let cur_page_table_addr = p.page_table_addr().start_address().as_u64();

        // set parent of children to parent of current process
        self.processes.iter_mut().for_each(|x| {
            if children.contains(&x.pid()) {
                x.set_parent(parent);
            }
        });

        // any process using the same page table as current process?
        let not_drop_page_table = self.processes.iter().any(|x| {
            x.page_table_addr().start_address().as_u64() == cur_page_table_addr && pid != x.pid()
        });

        // mark thr page table as not to be dropped
        if not_drop_page_table {
            self.pid_mut(pid).not_drop_page_table();
        }

        // remove process from process list
        self.processes.retain(|p| p.pid() != pid);

        // remove self from parent's children list
        if let Some(proc) = self.processes.iter_mut().find(|x| x.pid() == parent) {
            proc.remove_child(pid);
        }

        if ret == !0xdeadbeef {
            // killed by other process
            return;
        }

        if self.exit_code.try_insert(pid, ret).is_err() {
            error!("Process #{} already exited", pid);
        }
    }
}
