//! Types related to task management

use crate::timer::get_time_ms;

use super::TaskContext;
use alloc::collections::BTreeMap;

/// The task control block (TCB) of a task.
#[derive(Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// Syscall count
    pub syscall_cnt: BTreeMap<usize, u32>,
    /// Start time in ms
    pub birth: usize
}

impl TaskControlBlock {
    /// Constructor
    pub fn new() -> TaskControlBlock {
        TaskControlBlock { task_status: TaskStatus::UnInit, task_cx: TaskContext::zero_init(), syscall_cnt: BTreeMap::new(), birth: get_time_ms() }
    }
    /// Update syscall cnt
    pub fn update_cnt(&mut self, id: &usize) {
        let entry = self.syscall_cnt.entry(*id).or_insert(0);
        *entry += 1;
    }
    /// Get syscall cnt
    pub fn get_cnt(&self) -> &BTreeMap<usize, u32> {
        &self.syscall_cnt
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
