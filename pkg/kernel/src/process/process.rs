
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessState {
    Created,
    Running,
    Ready,
    Blocked,
    Dead
}

