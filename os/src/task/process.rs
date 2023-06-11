//! Implementation of  [`ProcessControlBlock`]

use super::id::RecycleAllocator;
use super::manager::insert_into_pid2process;
use super::TaskControlBlock;
use super::{add_task, SignalFlags};
use super::{pid_alloc, PidHandle};
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{translated_refmut, MemorySet, KERNEL_SPACE};
use crate::sync::{Condvar, Mutex, Semaphore, UPSafeCell};
use crate::trap::{trap_handler, TrapContext};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;

/// Process Control Block
pub struct ProcessControlBlock {
    /// immutable
    pub pid: PidHandle,
    /// mutable
    inner: UPSafeCell<ProcessControlBlockInner>,
}

/// Inner of Process Control Block
pub struct ProcessControlBlockInner {
    /// is zombie?
    pub is_zombie: bool,
    /// memory set(address space)
    pub memory_set: MemorySet,
    /// parent process
    pub parent: Option<Weak<ProcessControlBlock>>,
    /// children process
    pub children: Vec<Arc<ProcessControlBlock>>,
    /// exit code
    pub exit_code: i32,
    /// file descriptor table
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    /// signal flags
    pub signals: SignalFlags,
    /// tasks(also known as threads)
    pub tasks: Vec<Option<Arc<TaskControlBlock>>>,
    /// task resource allocator
    pub task_res_allocator: RecycleAllocator,
    /// mutex list
    pub mutex_list: Vec<Option<Arc<dyn Mutex>>>,
    /// semaphore list
    pub semaphore_list: Vec<Option<Arc<Semaphore>>>,
    /// condvar list
    pub condvar_list: Vec<Option<Arc<Condvar>>>,

    /// Below are implemented for ch8
    /// deadlock detect
    pub deadlock_detect: bool,
    /// available list
    pub available_list: Vec<usize>,
    /// allocation matrix
    pub allocation_matrix: Vec<Vec<usize>>,
    /// need matrix
    pub need_matrix: Vec<Vec<usize>>,
    /// map for mutex_id to resource_id
    pub mutex_id2res_id: BTreeMap<usize, usize>,
    /// map for semaphore_id to resource_id
    pub semaphore_id2res_id: BTreeMap<usize, usize>,
}

impl ProcessControlBlockInner {
    #[allow(unused)]
    /// get the address of app's page table
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    /// allocate a new file descriptor
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
    /// allocate a new task id
    pub fn alloc_tid(&mut self) -> usize {
        self.task_res_allocator.alloc()
    }
    /// deallocate a task id
    pub fn dealloc_tid(&mut self, tid: usize) {
        self.task_res_allocator.dealloc(tid)
    }
    /// the count of tasks(threads) in this process
    pub fn thread_count(&self) -> usize {
        self.tasks.len()
    }
    /// get a task with tid in this process
    pub fn get_task(&self, tid: usize) -> Arc<TaskControlBlock> {
        self.tasks[tid].as_ref().unwrap().clone()
    }
    /// update need matrix
    pub fn need(&mut self, thread_id:usize, res_id: usize) {
        self.need_matrix[thread_id][res_id] += 1;
    }
    /// detect deadlock:
    /// if deadlock detected, return true
    /// else return false
    pub fn deadlock_detected(&self) -> bool {
        if self.deadlock_detect {
            // TODO: detect deadlock
            let mut finish = vec![false; self.thread_count()];
            let mut work = self.available_list.clone();
            let mut finish_flag = true;

            while finish_flag {
                finish_flag = false;
                for i in 0..self.thread_count() {
                    if !finish[i] {
                        let mut flag = true;
                        for j in 0..self.available_list.len() {
                            if self.need_matrix[i][j] > work[j] {
                                flag = false;
                                break;
                            }
                        }
                        if flag {
                            finish_flag = true;
                            finish[i] = true;
                            for j in 0..self.available_list.len() {
                                work[j] += self.allocation_matrix[i][j];
                            }
                        }
                    }
                }
            }
            for i in 0..self.thread_count() {
                if !finish[i] {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }
    /// allocate a new resource index for mutex
    pub fn alloc_mutex_res_id(&mut self, mutex_id: usize) {
        // allocate a new resource id to this mutex
        // before allocation, check if this mutex has been allocated a resource id
        assert!(self.mutex_id2res_id.get(&mutex_id).is_none());
        // allocate a new resource id
        let res_id = self.available_list.len();
        self.mutex_id2res_id.insert(mutex_id, res_id);
        self.available_list.push(1);
        // as new resource is allocated, and no thread is accessing this resource
        // we should add a new column meaning that no thread is accessing this resource
        for i in 0..self.allocation_matrix.len() {
            self.allocation_matrix[i].push(0);
            self.need_matrix[i].push(0);
        }
    }  
    /// get resource index from mutex id
    pub fn get_mutex_res_id(&self, mutex_id: usize) -> usize {
        *self.mutex_id2res_id.get(&mutex_id).unwrap()
    }
    /// allocate a new resource index for semaphore
    pub fn alloc_semaphore_res_id(&mut self, semaphore_id: usize) {
        // allocate a new resource id to this semaphore
        // before allocation, check if this semaphore has been allocated a resource id
        assert!(self.semaphore_id2res_id.get(&semaphore_id).is_none());
        // allocate a new resource id
        let res_id = self.available_list.len();
        self.semaphore_id2res_id.insert(semaphore_id, res_id);
        self.available_list.push(1);
        // as new resource is allocated, and no thread is accessing this resource
        // we should add a new column meaning that no thread is accessing this resource
        for i in 0..self.allocation_matrix.len() {
            self.allocation_matrix[i].push(0);
            self.need_matrix[i].push(0);
        }
    }
    /// get resource index from semaphore id
    pub fn get_semaphore_res_id(&self, sem_id: usize) -> usize {
        *self.semaphore_id2res_id.get(&sem_id).unwrap()
    }
    /// allocate a new resource for a specific thread
    pub fn alloc_task_resource(&mut self, thread_id:usize, resource_id: usize) {
        assert!(self.need_matrix[thread_id][resource_id] <= self.available_list[resource_id]);
        self.available_list[resource_id] -= self.need_matrix[thread_id][resource_id];
        self.allocation_matrix[thread_id][resource_id] += self.need_matrix[thread_id][resource_id];
        self.need_matrix[thread_id][resource_id] = 0;
    }
    /// deallocate a resource for a specific thread
    pub fn dealloc_task_resource(&mut self, thread_id:usize, resource_id: usize, exit: bool) {
        if exit {
            for i in 0..self.available_list.len() {
                self.available_list[i] += self.allocation_matrix[thread_id][i];
                self.allocation_matrix[thread_id][i] = 0;
                self.need_matrix[thread_id][i] = 0;
            }
        }
        else {
            self.available_list[resource_id] += 1;
            if self.allocation_matrix[thread_id][resource_id] != 0 {
                self.allocation_matrix[thread_id][resource_id] -= 1;
            }
            self.need_matrix[thread_id][resource_id] = 0;
        }
    }
    /// add a new row for a new thread
    pub fn init_task_resource(&mut self, thread_id:usize) {
        // if this thread_id is new, we should add a new row for it
        println!("init_task_resource: thread_id = {}, allocation_matrix.len() = {}", thread_id, self.allocation_matrix.len());
        if self.allocation_matrix.len() <= thread_id {
            self.allocation_matrix.push(vec![0; self.available_list.len()]);
            self.need_matrix.push(vec![0; self.available_list.len()]);
            println!("init_task_resource: thread_id = {}, need_matrix.len() = {}", thread_id, self.need_matrix.len());
        } else {
            self.allocation_matrix[thread_id] = vec![0; self.available_list.len()];
            self.need_matrix[thread_id] = vec![0; self.available_list.len()];
        }
    }
}

impl ProcessControlBlock {
    /// inner_exclusive_access
    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
    /// new process from elf file
    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        trace!("kernel: ProcessControlBlock::new");
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        // allocate a pid
        let pid_handle = pid_alloc();
        let process = Arc::new(Self {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
                    semaphore_list: Vec::new(),
                    condvar_list: Vec::new(),

                    deadlock_detect: false,
                    available_list: Vec::new(),
                    allocation_matrix: Vec::new(),
                    need_matrix: Vec::new(),
                    mutex_id2res_id: BTreeMap::new(),
                    semaphore_id2res_id: BTreeMap::new(),
                })
            },
        });
        // create a main thread, we should allocate ustack and trap_cx here
        let task = Arc::new(TaskControlBlock::new(
            Arc::clone(&process),
            ustack_base,
            true,
        ));
        // prepare trap_cx of main thread
        let task_inner = task.inner_exclusive_access();
        let task_id = task_inner.res.as_ref().unwrap().tid;
        let trap_cx = task_inner.get_trap_cx();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = task.kstack.get_top();
        drop(task_inner);
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            ustack_top,
            KERNEL_SPACE.exclusive_access().token(),
            kstack_top,
            trap_handler as usize,
        );
        // add main thread to the process
        let mut process_inner = process.inner_exclusive_access();
        process_inner.tasks.push(Some(Arc::clone(&task)));
        // now row for main thread
        process_inner.init_task_resource(task_id);
        drop(process_inner);
        insert_into_pid2process(process.getpid(), Arc::clone(&process));
        // add main thread to scheduler
        add_task(task);
        process
    }

    /// Only support processes with a single thread.
    pub fn exec(self: &Arc<Self>, elf_data: &[u8], args: Vec<String>) {
        trace!("kernel: exec");
        assert_eq!(self.inner_exclusive_access().thread_count(), 1);
        // memory_set with elf program headers/trampoline/trap context/user stack
        trace!("kernel: exec .. MemorySet::from_elf");
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        let new_token = memory_set.token();
        // substitute memory_set
        trace!("kernel: exec .. substitute memory_set");
        self.inner_exclusive_access().memory_set = memory_set;
        // then we alloc user resource for main thread again
        // since memory_set has been changed
        trace!("kernel: exec .. alloc user resource for main thread again");
        let task = self.inner_exclusive_access().get_task(0);
        let mut task_inner = task.inner_exclusive_access();
        task_inner.res.as_mut().unwrap().ustack_base = ustack_base;
        task_inner.res.as_mut().unwrap().alloc_user_res();
        task_inner.trap_cx_ppn = task_inner.res.as_mut().unwrap().trap_cx_ppn();
        // push arguments on user stack
        trace!("kernel: exec .. push arguments on user stack");
        let mut user_sp = task_inner.res.as_mut().unwrap().ustack_top();
        user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_sp;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| {
                translated_refmut(
                    new_token,
                    (argv_base + arg * core::mem::size_of::<usize>()) as *mut usize,
                )
            })
            .collect();
        *argv[args.len()] = 0;
        for i in 0..args.len() {
            user_sp -= args[i].len() + 1;
            *argv[i] = user_sp;
            let mut p = user_sp;
            for c in args[i].as_bytes() {
                *translated_refmut(new_token, p as *mut u8) = *c;
                p += 1;
            }
            *translated_refmut(new_token, p as *mut u8) = 0;
        }
        // make the user_sp aligned to 8B for k210 platform
        user_sp -= user_sp % core::mem::size_of::<usize>();
        // initialize trap_cx
        trace!("kernel: exec .. initialize trap_cx");
        let mut trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            task.kstack.get_top(),
            trap_handler as usize,
        );
        trap_cx.x[10] = args.len();
        trap_cx.x[11] = argv_base;
        *task_inner.get_trap_cx() = trap_cx;
    }

    /// Only support processes with a single thread.
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        trace!("kernel: fork");
        let mut parent = self.inner_exclusive_access();
        assert_eq!(parent.thread_count(), 1);
        // clone parent's memory_set completely including trampoline/ustacks/trap_cxs
        let memory_set = MemorySet::from_existed_user(&parent.memory_set);
        // alloc a pid
        let pid = pid_alloc();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        // create child process pcb
        let child = Arc::new(Self {
            pid,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
                    semaphore_list: Vec::new(),
                    condvar_list: Vec::new(),

                    deadlock_detect: false,
                    available_list: parent.available_list.clone(),
                    allocation_matrix: parent.allocation_matrix.clone(),
                    need_matrix: parent.need_matrix.clone(),
                    mutex_id2res_id: parent.mutex_id2res_id.clone(),
                    semaphore_id2res_id: parent.semaphore_id2res_id.clone(),
                })
            },
        });
        // add child
        parent.children.push(Arc::clone(&child));
        // create main thread of child process
        let task = Arc::new(TaskControlBlock::new(
            Arc::clone(&child),
            parent
                .get_task(0)
                .inner_exclusive_access()
                .res
                .as_ref()
                .unwrap()
                .ustack_base(),
            // here we do not allocate trap_cx or ustack again
            // but mention that we allocate a new kstack here
            false,
        ));
        // attach task to child process
        let mut child_inner = child.inner_exclusive_access();
        child_inner.tasks.push(Some(Arc::clone(&task)));
        drop(child_inner);
        // modify kstack_top in trap_cx of this thread
        let task_inner = task.inner_exclusive_access();
        let trap_cx = task_inner.get_trap_cx();
        trap_cx.kernel_sp = task.kstack.get_top();
        drop(task_inner);
        insert_into_pid2process(child.getpid(), Arc::clone(&child));
        // add this thread to scheduler
        add_task(task);
        child
    }
    /// get pid
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}
