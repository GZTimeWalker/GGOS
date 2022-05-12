use super::*;
use crate::utils::Registers;
use alloc::format;
use alloc::vec::Vec;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::VirtAddr;
use x86_64::structures::paging::PhysFrame;
use xmas_elf::ElfFile;

once_mutex!(pub PROCESS_MANAGER: ProcessManager);
guard_access_fn! {
    pub get_process_manager(PROCESS_MANAGER: ProcessManager)
}

pub struct ProcessManager {
    /// pid of the current running process
    cur_pid: ProcessId,
    processes: Vec<Process>,
}

impl ProcessManager {
    pub fn new(init: Process) -> Self {
        let mut processes = Vec::<Process>::new();
        processes.push(init);
        Self {
            cur_pid: ProcessId(0),
            processes,
        }
    }

    fn current_mut(&mut self) -> &mut Process {
        self.processes
            .iter_mut()
            .find(|x| x.pid() == self.cur_pid)
            .unwrap()
    }

    pub fn current(&self) -> &Process {
        self.processes
            .iter()
            .find(|x| x.pid() == self.cur_pid)
            .unwrap()
    }

    pub fn save_current(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        let current = self.current_mut();
        if current.is_running() {
            current.tick();
            current.save(regs, sf);
        }
        // debug!("Paused process #{}", self.cur_pid);
    }

    fn get_next_pos(&self) -> usize {
        let next_pos = self
            .processes
            .iter()
            .position(|x| x.pid() == self.cur_pid)
            .unwrap_or(0)
            + 1;
        if next_pos >= self.processes.len() {
            0
        } else {
            next_pos
        }
    }

    pub fn still_alive(&self, pid: ProcessId) -> bool {
        self.processes.iter().any(|x| x.pid() == pid)
    }

    pub fn switch_next(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        let pos = self.get_next_pos();
        let p = &mut self.processes[pos];

        // debug!("Next process {} #{}", p.name(), p.pid());
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
        let proc = self.processes.get(0).unwrap();
        proc.page_table_addr()
    }

    pub fn spawn(
        &mut self,
        elf: &ElfFile,
        name: String,
        parent: ProcessId,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let mut p = Process::new(
            &mut *crate::memory::get_frame_alloc_for_sure(),
            name,
            parent,
            self.get_kernel_page_table(),
            proc_data,
        );
        p.pause();
        p.init_stack_frame(
            VirtAddr::new_truncate(elf.header.pt2.entry_point()),
            VirtAddr::new_truncate(STACK_TOP),
        );
        p.init_elf(elf);
        // info!("Spawn process: {}#{}", p.name(), p.pid());
        // info!("Spawn process:\n\n{:?}\n", p);
        let pid = p.pid();
        self.processes.push(p);
        pid
    }

    pub fn spawn_kernel_thread(
        &mut self,
        entry: VirtAddr,
        stack_top: VirtAddr,
        name: String,
        parent: ProcessId,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let mut p = Process::new(
            &mut *crate::memory::get_frame_alloc_for_sure(),
            name,
            parent,
            self.get_kernel_page_table(),
            proc_data,
        );
        p.pause();
        p.init_stack_frame(entry, stack_top);
        info!("Spawn process: {}#{}", p.name(), p.pid());
        let pid = p.pid();
        self.processes.push(p);
        pid
    }

    pub fn print_process_list(&self) {
        let mut output = String::from(" PID | Name       | Ticks\n");
        for p in self.processes.iter() {
            output = output + format!("{}\n", p).as_str();
        }
        print!("{}", output);
    }

    pub fn kill(&mut self) {
        self.processes.retain(|p| !p.is_running());
    }
}
