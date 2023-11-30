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
    ffi::CString,
    fs, mem, ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use ctor::ctor;
use dobby_api::{hook, resolve_func_addr, Address};
use libc::{c_int, c_void, size_t, ssize_t};

static mut LIBC_READ: Address = ptr::null_mut();
static mut FLAG: AtomicBool = AtomicBool::new(true);
static mut HOOKED: AtomicBool = AtomicBool::new(false);

#[ctor]
unsafe fn prepatch() {
    let addr = resolve_func_addr(None, "read").unwrap();
    hook(addr, patched_read as Address, Some(&mut LIBC_READ)).unwrap();
}

unsafe extern "C" fn patched_read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    let result = libc_read(fd, buf, count);

    if !FLAG.load(Ordering::Acquire) || HOOKED.load(Ordering::Acquire) {
        return result;
    }

    FLAG.store(false, Ordering::Release);

    let ppid = libc::getppid();
    if ppid != -1 && ppid != 1 {
        let pid = libc::getpid();
        let exe = format!("/proc/{pid}/exe");

        let Ok(bin_path) = fs::read_link(exe) else {
            FLAG.store(true, Ordering::Release);
            return result;
        };
        let bin_path = bin_path.parent().unwrap();

        let lib = bin_path.join("libmainpatch.so");
        let lib = lib.to_str().unwrap();
        let lib = CString::new(lib).unwrap();

        libc::dlopen(lib.as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL);
        HOOKED.store(true, Ordering::Release);
    }

    FLAG.store(true, Ordering::Release);

    result
}

unsafe fn libc_read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    let ori_read: extern "C" fn(c_int, *mut c_void, size_t) -> ssize_t = mem::transmute(LIBC_READ);
    ori_read(fd, buf, count)
}
