#![allow(clippy::missing_safety_doc)]
use std::mem;

use ctor::ctor;
use dobby_api::{hook, Address};
use libc::{c_int, c_void, size_t, ssize_t};

static mut LIBC_WRITE: Address = libc::write as Address;

#[ctor]
unsafe fn patch_main() {
    let _ = hook(
        libc::write as Address,
        write as Address,
        Some(&mut LIBC_WRITE),
    );
}

#[inline(always)]
pub unsafe extern "C" fn libc_write_ori(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    let ori_write: extern "C" fn(c_int, *const c_void, size_t) -> ssize_t =
        mem::transmute(LIBC_WRITE);
    ori_write(fd, buf, count)
}

pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    println!("a");
    libc_write_ori(fd, buf, count)
}
