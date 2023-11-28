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
use dobby_api::{hook, resolve_func_addr, Address};
use libc::{c_int, c_void, size_t, ssize_t};

use forward::{write_forward, WriteForward};

type Service = Strong<dyn IRemoteService::IRemoteService>;
type MutexService = Mutex<Service>;

static mut LIBC_WRITE: Address = ptr::null_mut();
pub(crate) static mut SERVICE: Option<Arc<MutexService>> = None;

#[ctor]
unsafe fn patch_main() {
    use IRemoteService::IRemoteService;

    loop {
        if let Ok(service) = binder::get_interface::<dyn IRemoteService>("fas_rs_server_uperf") {
            SERVICE = Some(Arc::new(Mutex::new(service)));
            break;
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }

    let addr = resolve_func_addr(None, "write").unwrap();
    hook(addr, patched_write as Address, Some(&mut LIBC_WRITE)).unwrap();

    let _ = SERVICE
        .clone()
        .unwrap()
        .lock()
        .unwrap()
        .connectServer();
}

unsafe extern "C" fn patched_write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    let ori_write: extern "C" fn(c_int, *const c_void, size_t) -> ssize_t =
        mem::transmute(LIBC_WRITE);

    match write_forward(fd, buf, count) {
        WriteForward::Allow => ori_write(fd, buf, count),
        WriteForward::Forward(s) => s,
    }
}
