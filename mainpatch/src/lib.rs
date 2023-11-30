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
#![allow(clippy::missing_safety_doc)]

mod IRemoteService;
mod forward;

use std::{
    mem, ptr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use binder::Strong;
use ctor::ctor;
use dobby_api::{hook, resolve_func_addr, undo_hook, Address};

use libc::{c_int, c_void, size_t, ssize_t};

use forward::{read_forward, write_forward, RwForward};

type Service = Strong<dyn IRemoteService::IRemoteService>;
type MutexService = Mutex<Service>;

static mut LIBC_WRITE: Address = ptr::null_mut();
static mut LIBC_READ: Address = ptr::null_mut();
static mut SERVICE: Option<Arc<MutexService>> = None;

#[ctor]
unsafe fn patch_main() {
    use IRemoteService::IRemoteService;

    SERVICE = loop {
        if let Ok(service) = binder::get_interface::<dyn IRemoteService>("fas_rs_server_uperf") {
            break Some(Arc::new(Mutex::new(service)));
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    };

    let addr = resolve_func_addr(None, "write").unwrap();
    let _ = undo_hook(addr);
    hook(addr, patched_write as Address, Some(&mut LIBC_WRITE)).unwrap();

    let addr = resolve_func_addr(None, "read").unwrap();
    let _ = undo_hook(addr);
    hook(addr, patched_read as Address, Some(&mut LIBC_READ)).unwrap();

    let _ = SERVICE.clone().unwrap().lock().unwrap().connectServer();
}

pub(crate) unsafe fn libc_write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    let ori_write: extern "C" fn(c_int, *const c_void, size_t) -> ssize_t =
        mem::transmute(LIBC_WRITE);
    ori_write(fd, buf, count)
}

pub(crate) unsafe fn libc_read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    let ori_read: extern "C" fn(c_int, *mut c_void, size_t) -> ssize_t = mem::transmute(LIBC_READ);
    ori_read(fd, buf, count)
}

unsafe extern "C" fn patched_write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    match write_forward(fd, buf, count) {
        RwForward::Allow => libc_write(fd, buf, count),
        RwForward::Forward(s) => s,
    }
}

unsafe extern "C" fn patched_read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    match read_forward(fd, buf, count) {
        RwForward::Allow => libc_read(fd, buf, count),
        RwForward::Forward(s) => s,
    }
}
