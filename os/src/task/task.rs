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
    /// Syscall count
    pub syscall_cnt: [u32;MAX_SYSCALL_NUM],
    /// Start time in ms
    pub birth: usize
}

impl TaskControlBlock {
    /// Update syscall cnt
    pub fn update_cnt(&mut self, id: &usize) {
        self.syscall_cnt[*id] += 1;
    }
    /// Get syscall cnt
    pub fn get_cnt(&self) -> [u32;MAX_SYSCALL_NUM]{
        self.syscall_cnt
    }
    /// Get birth timestamp
    pub fn get_birth(&self) -> usize {
        self.birth
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
