//! sqlite-vec 自动扩展的进程级注册（chat.db / knowledge.db 共用）。

use rusqlite::ffi::sqlite3_auto_extension;
use std::sync::Once;

/// Register sqlite-vec as a process-wide SQLite auto-extension once.
///
/// `sqlite3_auto_extension` runs the callback on every new connection, so the
/// `vec0` vtable is available on any rusqlite connection opened after this call.
/// Must run before `Connection::open`.
pub fn register_vec_extension() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut libsqlite3_sys::sqlite3,
                *mut *mut i8,
                *const libsqlite3_sys::sqlite3_api_routines,
            ) -> i32,
        >(sqlite_vec::sqlite3_vec_init as *const ())));
    });
}
