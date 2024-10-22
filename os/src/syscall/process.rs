//! Process management syscalls

use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, get_current_task, suspend_current_and_run_next, TaskStatus, APP_TIME},
    //config::MAX_APP_NUM,
    timer::{get_time_ms, get_time_us},
    syscall::APP_SYSCALL_TIMES,
};
//use lazy_static::*;
//use crate::sync::UPSafeCell;


#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}
#[derive(Copy,Clone)]
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
/*
///
pub struct TaskInfoWrapper {
    task_infos: UPSafeCell<[TaskInfo; MAX_APP_NUM]>,
}

impl TASK_INFO_WRAPPER {
    fn syscall_times(&self,id: usize){
        let mut task_infos = self.task_infos.exclusive_access();
        let task_curr = &mut task_infos[get_current_task()];
        task_curr.syscall_times[id] += 1;
    }
}

///
pub fn syscall_times(id: usize){
    TASK_INFO_WRAPPER.syscall_times(id);
}

lazy_static! {
    /// Global variable: TASK_INFO_WRAPPER
    pub static ref TASK_INFO_WRAPPER: TaskInfoWrapper = {
        let task_infos = unsafe {
            UPSafeCell::new([ 
                TaskInfo {
                    status: TaskStatus::UnInit,
                    syscall_times: [0; MAX_SYSCALL_NUM],
                    time: 0,
                }; // Repeat for MAX_APP_NUM
                MAX_APP_NUM// This creates an array of MAX_APP_NUM TaskInfo instances, all initialized.
            ])
            
        };
        TaskInfoWrapper {
            task_infos,
        }
    };
}
 */
/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    
    unsafe {
        (*_ti).syscall_times = APP_SYSCALL_TIMES[get_current_task()];
        (*_ti).status = TaskStatus::Running;
        (*_ti).time =  get_time_ms() - APP_TIME;
        //print_task_info_by_id(&*_ti);
    }
    0
}
/* 
fn print_task_info_by_id(task_info: &TaskInfo) {
    // 直接使用 println! 打印 TaskInfo 的所有信息
    println!("TaskInfo:");
    println!("  Status: {}", task_info.status as usize); // 将状态转换为 usize
    for (id, times) in task_info.syscall_times.iter().enumerate() {
        println!("    Syscall ID: {}, Times: {}", id, times);
    }
    println!("  Time: {}", task_info.time);
}
*/
