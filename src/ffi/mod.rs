//! C-compatible FFI layer for using rufdb from Node.js, Python, Ruby, etc.
//!
//! This module exposes a JSON-in/JSON-out C API that any language with FFI
//! support can call. The database stores arbitrary JSON (via `serde_json::Value`).
//!
//! # Usage from C / shared library consumers
//!
//! ```c
//! void* db = rufdb_open("db.json", "{\"posts\":[]}");
//! rufdb_read(db);
//! const char* json = rufdb_get_data(db);
//! rufdb_set_data(db, "{\"posts\":[{\"title\":\"hello\"}]}");
//! rufdb_write(db);
//! rufdb_free_string(json);
//! rufdb_close(db);
//! ```
//!
//! # Building
//!
//! Build with `cargo build --release --features ffi` to produce:
//! - `librufdb.dylib` (macOS) / `librufdb.so` (Linux) / `rufdb.dll` (Windows)
//! - `librufdb.a` (static)
//!
//! # Safety
//!
//! All functions that take a `*mut DbHandle` or `*const DbHandle` return an
//! error code (or null) if a null pointer is passed. Callers must still ensure
//! the pointer was obtained from `rufdb_open` and has not been freed via
//! `rufdb_close`.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::adapters::JSONFileSync;
use crate::core::low::LowSync;

/// Opaque handle to a database instance.
type DbHandle = LowSync<serde_json::Value, JSONFileSync<serde_json::Value>>;

/// Open a database. Returns an opaque pointer.
///
/// - `path`: null-terminated UTF-8 path to the JSON file
/// - `default_data_json`: null-terminated UTF-8 JSON string for default data
///
/// Returns null on error (including null input pointers or invalid JSON).
#[no_mangle]
pub extern "C" fn rufdb_open(
    path: *const c_char,
    default_data_json: *const c_char,
) -> *mut DbHandle {
    if path.is_null() || default_data_json.is_null() {
        return std::ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let default_json = match unsafe { CStr::from_ptr(default_data_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let default_data: serde_json::Value = match serde_json::from_str(default_json) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    let adapter = JSONFileSync::new(path);
    let db = LowSync::new(adapter, default_data);
    Box::into_raw(Box::new(db))
}

/// Read data from disk into the database.
/// Returns 0 on success, -1 on error (including null pointer).
#[no_mangle]
pub extern "C" fn rufdb_read(db: *mut DbHandle) -> i32 {
    if db.is_null() {
        return -1;
    }
    let db = unsafe { &mut *db };
    match db.read() {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Write current data to disk.
/// Returns 0 on success, -1 on error (including null pointer).
#[no_mangle]
pub extern "C" fn rufdb_write(db: *mut DbHandle) -> i32 {
    if db.is_null() {
        return -1;
    }
    let db = unsafe { &*db };
    match db.write() {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Get the current data as a JSON string.
///
/// The returned pointer must be freed with `rufdb_free_string`.
/// Returns null on error (including null input pointer).
#[no_mangle]
pub extern "C" fn rufdb_get_data(db: *const DbHandle) -> *mut c_char {
    if db.is_null() {
        return std::ptr::null_mut();
    }
    let db = unsafe { &*db };
    match serde_json::to_string_pretty(&db.data) {
        Ok(s) => match CString::new(s) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Set the database data from a JSON string.
/// Returns 0 on success, -1 on error (including null pointers or invalid JSON).
#[no_mangle]
pub extern "C" fn rufdb_set_data(db: *mut DbHandle, json: *const c_char) -> i32 {
    if db.is_null() || json.is_null() {
        return -1;
    }
    let db = unsafe { &mut *db };
    let json_str = match unsafe { CStr::from_ptr(json) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match serde_json::from_str(json_str) {
        Ok(v) => {
            db.data = v;
            0
        }
        Err(_) => -1,
    }
}

/// Update data using a JSON Merge Patch (RFC 7396).
///
/// Merges the given JSON into the existing data, then writes to disk.
/// Returns 0 on success, -1 on error (including null pointers or invalid JSON).
#[no_mangle]
pub extern "C" fn rufdb_update(db: *mut DbHandle, patch_json: *const c_char) -> i32 {
    if db.is_null() || patch_json.is_null() {
        return -1;
    }
    let db = unsafe { &mut *db };
    let patch_str = match unsafe { CStr::from_ptr(patch_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let patch: serde_json::Value = match serde_json::from_str(patch_str) {
        Ok(v) => v,
        Err(_) => return -1,
    };

    json_merge_patch(&mut db.data, &patch);

    match db.write() {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Free a string returned by `rufdb_get_data`.
#[no_mangle]
pub extern "C" fn rufdb_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Close and free the database handle.
#[no_mangle]
pub extern "C" fn rufdb_close(db: *mut DbHandle) {
    if !db.is_null() {
        unsafe {
            let _ = Box::from_raw(db);
        }
    }
}

/// JSON Merge Patch (RFC 7396) implementation.
fn json_merge_patch(target: &mut serde_json::Value, patch: &serde_json::Value) {
    if let serde_json::Value::Object(patch_obj) = patch {
        if !target.is_object() {
            *target = serde_json::Value::Object(serde_json::Map::new());
        }
        if let serde_json::Value::Object(target_obj) = target {
            for (key, value) in patch_obj {
                if value.is_null() {
                    target_obj.remove(key);
                } else if value.is_object() {
                    let entry = target_obj
                        .entry(key.clone())
                        .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                    json_merge_patch(entry, value);
                } else {
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}
