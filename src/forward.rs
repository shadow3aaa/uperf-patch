/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
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

        let path = path.replace("scaling_min_freq", "scaling_max_freq");

        let _ = SERVICE
            .clone()
            .unwrap()
            .lock()
            .unwrap()
            .writeFile(&path, value);
        WriteForward::Forward(path.len() as ssize_t)
    } else {
        WriteForward::Allow
    }
}
