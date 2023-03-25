//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;
/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,

    /// The syscall count of task
    pub syscall_count:[u32;MAX_SYSCALL_NUM],
}

impl TaskControlBlock {
    /// update syscall count
    pub fn update_syscall_count(&mut self, syscall_id: usize) -> usize {
        self.syscall_count[syscall_id] += 1;
        syscall_id
    }
    /// get syscall count
    pub fn get_syscall_count(&self) -> [u32;MAX_SYSCALL_NUM] {
        self.syscall_count
    }
}


/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
