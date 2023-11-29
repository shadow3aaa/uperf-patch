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
    collections::HashMap,
    ffi::{CStr, CString},
    fs,
    path::{Path, PathBuf},
    ptr,
    sync::Mutex,
};

use lazy_static::lazy_static;
use libc::{c_char, c_int, c_void, size_t, ssize_t};

use crate::SERVICE;

lazy_static! {
    static ref TROLL_MAP: Mutex<HashMap<PathBuf, String>> = Mutex::new(HashMap::new());
}

pub enum RwForward {
    Allow,
    Forward(ssize_t),
}

pub unsafe fn write_forward(fd: c_int, buf: *const c_void, count: size_t) -> RwForward {
    let fd = format!("/proc/{}/fd/{fd}", libc::getpid());
    let fd = CString::new(fd).unwrap();

    let path = libc::realpath(fd.as_ptr(), ptr::null_mut());

    if path.is_null() {
        return RwForward::Allow;
    }

    let path = CStr::from_ptr(path).to_str().unwrap();

    if path.contains("scaling_governor") {
        RwForward::Forward(count as ssize_t)
    } else if path.contains("scaling_min_freq") || path.contains("scaling_max_freq") {
        let Ok(value) = CStr::from_ptr(buf as *const c_char).to_str() else {
            return RwForward::Forward(count as ssize_t);
        };

        let path = path.replace("scaling_min_freq", "scaling_max_freq");

        let _ = SERVICE
            .clone()
            .unwrap()
            .lock()
            .unwrap()
            .writeFile(&path, value);

        let path = Path::new(&path);
        TROLL_MAP
            .lock()
            .unwrap()
            .insert(path.parent().unwrap().to_path_buf(), value.to_string());

        let len = libc::strlen(buf as *const c_char);

        RwForward::Forward(len as ssize_t)
    } else {
        RwForward::Allow
    }
}

pub unsafe fn read_forward(fd: c_int, buf: *mut c_void, _count: size_t) -> RwForward {
    let path = format!("/proc/{}/fd/{fd}", libc::getpid());
    let path = CString::new(path).unwrap();

    let path = libc::realpath(path.as_ptr(), ptr::null_mut());

    if path.is_null() {
        return RwForward::Allow;
    }

    let path = CStr::from_ptr(path).to_str().unwrap();

    if path.contains("perapp_powermode.txt") {
        if let Ok(mode) = fs::read_to_string("/dev/fas_rs/mode") {
            let mode = CString::new(mode.trim()).unwrap();
            let mode = mode.as_ptr();

            let len = libc::strlen(mode) + 1;
            libc::strcpy(buf as *mut c_char, mode);

            return RwForward::Forward(len as ssize_t);
        }
    } else if path.contains("cur_freq") {
        let path = Path::new(path);
        let path = path.parent().unwrap();

        if let Some(value) = TROLL_MAP.lock().unwrap().get(path) {
            let value = CString::new(value.to_string()).unwrap();
            let value = value.as_ptr();

            let len = libc::strlen(value) + 1;
            libc::strcpy(buf as *mut c_char, value);

            return RwForward::Forward(len as ssize_t);
        }
    }

    RwForward::Allow
}
