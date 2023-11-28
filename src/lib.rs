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

    SERVICE
        .clone()
        .unwrap()
        .lock()
        .unwrap()
        .connectServer()
        .unwrap();
}

unsafe extern "C" fn patched_write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    let ori_write: extern "C" fn(c_int, *const c_void, size_t) -> ssize_t =
        mem::transmute(LIBC_WRITE);

    match write_forward(fd, buf, count) {
        WriteForward::Allow => ori_write(fd, buf, count),
        WriteForward::Forward(s) => s,
    }
}
