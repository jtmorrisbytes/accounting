/// ALLOCATOR API even though they are PUB, they are not meant to be called by rust code
/// only by the underlying library (zig)
/// prefer core when possible to remain compatible with no_std environemnts
use std::alloc::{Layout, alloc, dealloc, realloc};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

#[repr(C, align(8))]
struct Header {
    size: usize,
}
const H_SIZE: usize = std::mem::size_of::<Header>(); // Likely 8 or 16

#[unsafe(no_mangle)]
pub extern "C" fn __rust_jtmb_sqlite_alloc_malloc(n: i32) -> *mut core::ffi::c_void {
    if n <= 0 {
        return ptr::null_mut();
    }
    let n = n as usize;
    let total = n + H_SIZE;
    unsafe {
        // SQLite requires 8-byte alignment
        let layout = Layout::from_size_align_unchecked(total, 8);
        let p = alloc(layout);
        if p.is_null() {
            return ptr::null_mut();
        }

        // Write header and return pointer shifted forward
        *(p as *mut Header) = Header { size: total };
        p.add(H_SIZE).cast()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn __rust_jtmb_sqlite_alloc_free(p: *mut core::ffi::c_void) {
    if p.is_null() {
        return;
    }
    unsafe {
        let raw = p.sub(H_SIZE);
        let size = (*(raw as *const Header)).size;
        dealloc(raw.cast(), Layout::from_size_align_unchecked(size, 8));
    }
}
#[unsafe(no_mangle)]

pub extern "C" fn __rust_jtmb_sqlite_alloc_realloc(
    p: *mut core::ffi::c_void,
    n: i32,
) -> *mut core::ffi::c_void {
    unsafe {
        if p.is_null() {
            return __rust_jtmb_sqlite_alloc_malloc(n);
        }
        if n <= 0 {
            __rust_jtmb_sqlite_alloc_free(p);
            return ptr::null_mut();
        }

        let old_raw = p.sub(H_SIZE);
        let old_size = (*(old_raw as *const Header)).size;
        let new_total = (n as usize) + H_SIZE;

        let new_raw = realloc(
            old_raw.cast(),
            Layout::from_size_align_unchecked(old_size, 8),
            new_total,
        );
        if new_raw.is_null() {
            return ptr::null_mut();
        }

        *(new_raw as *mut Header) = Header { size: new_total };
        new_raw.add(H_SIZE).cast()
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn __rust_jtmb_sqlite_alloc_size(p: *mut core::ffi::c_void) -> i32 {
    unsafe {
        if p.is_null() {
            return 0;
        }
        let raw = p.sub(H_SIZE);
        let total_size = (*(raw as *const Header)).size;
        (total_size - H_SIZE) as i32
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn __rust_jtmb_sqlite_alloc_roundup(n: i32) -> i32 {
    (n + 7) & !7
}
#[unsafe(no_mangle)]
pub extern "C" fn __rust_jtmb_sqlite_alloc_init(p: *mut core::ffi::c_void) -> i32 {
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn __rust_jtmb_sqlite_alloc_shutdown(p: *mut core::ffi::c_void) {
    // do nothing
}

// Global pointer so Zig can "find" it if needed, or just pass it once.
static PAGE_CACHE_PTR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __rust_jtmb_sqlite_alloc_get_page_cache_buffer(
    out_size: *mut usize,
) -> *mut u8 {
    unsafe {
        let size = 64 * 1024 * 1024; // 64MB Page Cache
        let layout = Layout::from_size_align_unchecked(size, 8);

        let ptr = alloc(layout);
        if !ptr.is_null() {
            *out_size = size;
            PAGE_CACHE_PTR.store(ptr, Ordering::SeqCst);
        }
        ptr
    }
}
pub fn get_mem_methods() -> crate::ffi::sqlite3_mem_methods {
    return crate::ffi::sqlite3_mem_methods {
        xMalloc : Some(__rust_jtmb_sqlite_alloc_malloc),
        xFree : Some(__rust_jtmb_sqlite_alloc_free),
        xRealloc : Some(__rust_jtmb_sqlite_alloc_realloc),
        xSize : Some(__rust_jtmb_sqlite_alloc_size),
        xRoundup :Some( __rust_jtmb_sqlite_alloc_roundup),
        xInit : Some(__rust_jtmb_sqlite_alloc_init),
        xShutdown :Some( __rust_jtmb_sqlite_alloc_shutdown),
        pAppData : core::ptr::null_mut(),
    };
}
