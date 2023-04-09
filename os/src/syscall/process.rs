//! Process management syscalls
use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE, MEMORY_END},
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, current_task_create_time, current_task_syscall_time, mmap, munmap, current_user_token,
    }, timer::get_time_us, 
    mm::va2pa,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // let curr_time_us = get_time_us();
    // unsafe {
    //     (*_ts).sec = curr_time_us / 1000000;
    //     (*_ts).usec = curr_time_us % 1000000;
    // };

    // Current mode : kernel
    // _ts is VirtAddr from user mode
    // to get correct task start time, we nned to convert it to PhysAddr and then
    // access is to get the value
    let curr_time_us = get_time_us();
    *va2pa(current_user_token(), _ts) = TimeVal {
        sec: curr_time_us / 1000000,
        usec: curr_time_us % 1000000,
    };
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let curr_time_us = get_time_us();
    let curr_task_create_time = current_task_create_time();
    let syscall_cnt = current_task_syscall_time();
    let intercal = (curr_time_us - curr_task_create_time) / 1000;
    let mut syscall_times = [0; MAX_SYSCALL_NUM];
    for (key, value) in syscall_cnt {
        syscall_times[key] = value;
    }
    *va2pa(current_user_token(), _ti) = TaskInfo {
        status: TaskStatus::Running,
        syscall_times,
        time: intercal,
    };
    0
}

// YOUR JOB: Implement mmap.
// start 没有按页大小对齐
// port & !0x7 != 0 (port 其余位必须为0)
// port & 0x7 = 0 (这样的内存无意义)
// [start, start + len) 中存在已经被映射的页
// 物理内存不足
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    if _port & !0x7 != 0 {
        return -1;
    }
    if _port & 0x7 != 0 {
        return -1;
    }
    if _len == 0 || _start + _len > MEMORY_END {
        return -1;
    }
    mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
// [start, start + len) 中存在未被映射的虚存。
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    if _len == 0 || _start + _len > MEMORY_END {
        return -1;
    }
    munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
