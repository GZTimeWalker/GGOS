use super::*;
use alloc::vec::Vec;
use crate::utils::Registers;
use x86_64::VirtAddr;
use x86_64::structures::idt::InterruptStackFrame;

once_mutex!(pub PROCESS_MANAGER: ProcessManager);
guard_access_fn! {
    pub get_process_manager(PROCESS_MANAGER: ProcessManager)
}

pub struct ProcessManager {
    /// the next pid to be assigned
    next_pid: u16,
    /// pid of the current running process
    cur_pid: u16,
    processes: Vec<Process>
}

impl ProcessManager {
    pub fn new(init: Process) -> Self{
        let mut processes = Vec::<Process>::new();
        processes.push(init);
        Self {
            cur_pid: 0,
            next_pid: 1,
            processes
        }
    }

    fn get_current(&mut self) -> &mut Process {
        self.processes.iter_mut().find(|x| x.pid() == self.cur_pid).unwrap()
    }

    pub fn tick(&mut self) -> bool {
        self.get_current().tick()
    }

    pub fn save_current(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        let current = self.get_current();
        if current.is_running() {
            current.save(regs, sf);
        }
        // trace!("Paused process #{}", self.cur_pid);
    }

    fn get_next_pos(&self) -> usize {
        let next_pos = self.processes.iter()
            .position(|x| x.pid() == self.cur_pid).unwrap() + 1;
        if next_pos >= self.processes.len() {
            0
        } else {
            next_pos
        }
    }

    pub fn switch_next(&mut self, regs: &mut Registers, sf: &mut InterruptStackFrame) {
        let pos = self.get_next_pos();
        let p = &mut self.processes[pos];

        // trace!("Next process {}#{}", p.name(), p.pid());
        if p.pid() == self.cur_pid {
            // the next process to be resumed is the same as the current one
            p.resume();
        } else {
            // switch to next process
            p.restore(regs, sf);
            self.cur_pid = p.pid();
        }
    }

    pub fn spawn(&mut self, entry: VirtAddr, stack_top: VirtAddr,
            name: String, priority: usize, parent: u16) -> u16{
        let mut p = Process::new(
            &mut *crate::memory::get_frame_alloc_for_sure(),
            self.next_pid, name, priority, parent
        );
        p.pause();
        p.init_stack_frame(entry, stack_top);
        let pid = p.pid();
        self.processes.push(p);
        self.next_pid += 1; // TODO: recycle PID
        pid
    }

    pub fn kill(&mut self) {
        self.processes.retain(|p| !p.is_running());
    }
}
