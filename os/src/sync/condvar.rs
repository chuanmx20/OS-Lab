//! Conditian variable

use crate::sync::{Mutex, UPSafeCell};
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
use alloc::{collections::VecDeque, sync::Arc};

/// Condition variable structure
pub struct Condvar {
    /// Condition variable inner
    pub inner: UPSafeCell<CondvarInner>,
}

pub struct CondvarInner {
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Condvar {
    /// Create a new condition variable
    pub fn new() -> Self {
        trace!("kernel: Condvar::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(CondvarInner {
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// Signal a task waiting on the condition variable
    pub fn signal(&self) {
        let mut inner = self.inner.exclusive_access();
        if let Some(task) = inner.wait_queue.pop_front() {
            wakeup_task(task);
        }
    }

    /// blocking current task, let it wait on the condition variable
    pub fn wait(&self, mutex: Arc<dyn Mutex>, mutex_id: usize) -> isize {
        trace!("kernel: Condvar::wait_with_mutex");
        mutex.unlock(mutex_id);
        let mut inner = self.inner.exclusive_access();
        inner.wait_queue.push_back(current_task().unwrap());
        drop(inner);
        block_current_and_run_next();

        // now alloc mutex resource
        let process = current_task().unwrap().process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        let resource_id = process_inner.get_mutex_res_id(mutex_id);
        let task = current_task().unwrap();
        let task_inner = task.inner_exclusive_access();
        let task_id = task_inner.res.as_ref().unwrap().tid;
        process_inner.need(task_id, resource_id);
        if process_inner.deadlock_detected() {
            return -0xDEAD;
        }
        drop(process_inner);
        drop(process);
        drop(task_inner);
        mutex.lock(mutex_id);
        0
    }
}
