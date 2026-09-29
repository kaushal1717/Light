//! The process's global allocator: a plain `System` pass-through until `arm` is called, after
//! which any allocation that would grow the heap past the cap fails -- and Rust aborts the
//! process on a failed allocation. Only the hidden `fleet __pdf-text` child arms it, so a PDF
//! whose streams inflate to gigabytes kills that throwaway child, never the CLI that spawned it
//! (`dispatch/run/pdf_text_cmd.rs`). Unarmed, the only cost is one relaxed atomic load.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering::Relaxed};

/// Cap on net heap growth since `arm`, in bytes; `0` means unarmed.
static CAP: AtomicIsize = AtomicIsize::new(0);
/// Net bytes since `arm`; frees of pre-`arm` memory only ever loosen the cap, never tighten it.
static USED: AtomicIsize = AtomicIsize::new(0);

struct Capped;
#[global_allocator]
static GLOBAL: Capped = Capped;

pub fn arm(cap_bytes: usize) {
    USED.store(0, Relaxed);
    CAP.store(isize::try_from(cap_bytes).unwrap_or(isize::MAX), Relaxed);
}

fn admit(bytes: usize) -> bool {
    let cap = CAP.load(Relaxed);
    if cap == 0 {
        return true;
    }
    let bytes = isize::try_from(bytes).unwrap_or(isize::MAX);
    let before = USED.fetch_add(bytes, Relaxed);
    if before.checked_add(bytes).is_none_or(|after| after > cap) {
        USED.fetch_sub(bytes, Relaxed);
        return false;
    }
    true
}

fn release(bytes: usize) {
    if CAP.load(Relaxed) != 0 {
        USED.fetch_sub(isize::try_from(bytes).unwrap_or(isize::MAX), Relaxed);
    }
}

/// Admit `bytes` against the cap, then allocate; an allocation the system itself refuses gives
/// its admission back, so `USED` tracks only memory actually held.
fn guarded(bytes: usize, allocate: impl FnOnce() -> *mut u8) -> *mut u8 {
    if !admit(bytes) {
        return std::ptr::null_mut();
    }
    let ptr = allocate();
    if ptr.is_null() {
        release(bytes);
    }
    ptr
}

unsafe impl GlobalAlloc for Capped {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        guarded(layout.size(), || unsafe { System.alloc(layout) })
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        guarded(layout.size(), || unsafe { System.alloc_zeroed(layout) })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        release(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new = guarded(new_size.saturating_sub(layout.size()), || unsafe {
            System.realloc(ptr, layout, new_size)
        });
        if !new.is_null() {
            release(layout.size().saturating_sub(new_size));
        }
        new
    }
}
