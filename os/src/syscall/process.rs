//! Process management syscalls
use crate::{
    bitflags::bitflags, config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translate_to_phys_addr, MapPermission, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_start_time, suspend_current_and_run_next, TaskStatus
    }, timer::{get_time_ms,get_time_us},
};
use crate::task::{get_syscall_times,syscall_mmap,syscall_munmap};

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
    let token = current_user_token();
    let phys_addr:usize= translate_to_phys_addr(
        token,
        _ts  as usize
    );
    let us = get_time_us();
    unsafe {
        *(phys_addr as *mut TimeVal) = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let token = current_user_token();
    let phys_addr= translate_to_phys_addr(
        token,
        _ti as usize
    );
    let ptr = phys_addr as *mut TaskInfo;
    unsafe {
        (*ptr).syscall_times = get_syscall_times();
        (*ptr).status = TaskStatus::Running;
        (*ptr).time =  get_time_ms() - get_start_time();
    }
    0
}

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct SysMmapPermission: u8 {
        const R = 1;
        const W = 1 << 1;
        const X = 1 << 2;
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // 检查参数合法性
    if _len == 0 || _start % PAGE_SIZE != 0 {
        return -1; // 非法的 `len` 或 `start` 地址不对齐
    }

    // 检查 prot 是否只有前三位有效
    if _port & !0b111 != 0 {
        return -1; // prot 包含无效位，其他位必须为 0
    }
    if _port & 0b111 == 0{
        return -1;
    }

    // 将 `prot` 参数转换为 `SysMmapPermission` 标志
    let permissions = SysMmapPermission::from_bits(_port as u8).unwrap();
    // 转换为 `MapPermission`
    let map_permissions = convert_sysmmap_to_map_permission(permissions);

    syscall_mmap(_start,_len,map_permissions);
    0
}

/// 将 `SysMmapPermission` 转换为 `MapPermission`
#[allow(unused)]
fn convert_sysmmap_to_map_permission(permissions: SysMmapPermission) -> MapPermission {
    let mut map_perm = MapPermission::empty();
    if permissions.contains(SysMmapPermission::R) {
        map_perm |= MapPermission::R;
    }
    if permissions.contains(SysMmapPermission::W) {
        map_perm |= MapPermission::W;
    }
    if permissions.contains(SysMmapPermission::X) {
        map_perm |= MapPermission::X;
    }
    map_perm | MapPermission::U // 用户权限标志
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {

     // 检查 start 是否页对齐
    if _start % PAGE_SIZE != 0 {
        return -1; // 非法的 start 地址
    }

    let start_va: VirtAddr = _start.into();
    let end_va: VirtAddr = (_start+_len).into();
    if  !start_va.aligned() || !end_va.aligned(){
        return -1;
    }
    syscall_munmap(_start, _len);
    0
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
