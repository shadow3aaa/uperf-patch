use std::{
    ffi::{CStr, CString},
    ptr,
};

use libc::{c_int, c_void, size_t, ssize_t};

use crate::SERVICE;

pub enum WriteForward {
    Allow,
    Forward(ssize_t),
}

pub unsafe fn write_forward(fd: c_int, buf: *const c_void, _count: size_t) -> WriteForward {
    let fd = format!("/proc/{}/fd/{fd}", libc::getpid());
    let fd = CString::new(fd).unwrap();

    let path = libc::realpath(fd.as_ptr(), ptr::null_mut());

    if path.is_null() {
        return WriteForward::Allow;
    }

    let path = CStr::from_ptr(path).to_str().unwrap();

    if path.contains("scaling_governor") {
        WriteForward::Forward(path.len() as ssize_t)
    } else if path.contains("scaling_min_freq") || path.contains("scaling_max_freq") {
        let Ok(value) = CStr::from_ptr(buf as *const u8).to_str() else {
            return WriteForward::Forward(path.len() as ssize_t);
        };

        SERVICE
            .clone()
            .unwrap()
            .lock()
            .unwrap()
            .writeFile(path, value)
            .unwrap();
        WriteForward::Forward(path.len() as ssize_t)
    } else {
        WriteForward::Allow
    }
}
