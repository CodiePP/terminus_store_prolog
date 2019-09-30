use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::io;

use terminus_store::storage::{
    DirectoryLabelStore, DirectoryLayerStore,
};
use terminus_store::layer::{
    StringTriple, IdTriple
};
use terminus_store::sync::store::*;

#[no_mangle]
pub unsafe extern "C" fn open_directory_store(
    dir: *const c_char,
) -> *const SyncStore<DirectoryLabelStore, DirectoryLayerStore> {
    // Safe because swipl will always return a null-terminated string
    let dir_name_cstr = CStr::from_ptr(dir);
    let dir_name = dir_name_cstr.to_str().unwrap();
    let store = open_sync_directory_store(dir_name);
    Box::into_raw(Box::new(store))
}

fn error_to_cstring(error: io::Error) -> CString {
    CString::new(format!("{}", error)).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn create_database(
    store_ptr: *mut c_void,
    name: *const c_char,
    err: *mut *const c_char,
) -> *const SyncDatabase<DirectoryLabelStore, DirectoryLayerStore> {
    let store = store_ptr as *mut SyncStore<DirectoryLabelStore, DirectoryLayerStore>;
    let store_box = Box::from_raw(store);
    // We assume it to be somewhat safe because swipl will check string types
    let db_name_cstr = CStr::from_ptr(name);
    let db_name = db_name_cstr.to_str().unwrap();

    let result = store_box.create(db_name);
    std::mem::forget(store_box);
    // Safe because we expect the swipl pointers to be decent
    match result {
        Ok(database) => {
            Box::into_raw(Box::new(database))
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn open_database(
    store_ptr: *mut c_void,
    name: *const c_char,
    err: *mut *const c_char,
) -> *const SyncDatabase<DirectoryLabelStore, DirectoryLayerStore> {
    let store = store_ptr as *mut SyncStore<DirectoryLabelStore, DirectoryLayerStore>;
    let store_box = Box::from_raw(store);
    // We assume it to be somewhat safe because swipl will check string types
    let db_name_cstr = CStr::from_ptr(name);
    let db_name = db_name_cstr.to_str().unwrap();

    let result = store_box.open(db_name);
    std::mem::forget(store_box);
    // Safe because we expect the swipl pointers to be decent
    match result {
        Ok(Some(database)) => {
            *err = std::ptr::null();
            Box::into_raw(Box::new(database))
        }
        Ok(None) => {
            *err = std::ptr::null();
            std::ptr::null()
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn database_get_head(database_ptr: *mut SyncDatabase<DirectoryLabelStore, DirectoryLayerStore>, err: *mut *const c_char) -> *const SyncDatabaseLayer<DirectoryLayerStore> {
    let database_box = Box::from_raw(database_ptr);
    let result = match database_box.head() {
        Ok(None) => {
            *err = std::ptr::null();
            std::ptr::null()
        },
        Ok(Some(layer)) => {
            *err = std::ptr::null();
            Box::into_raw(Box::new(layer))
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            std::ptr::null()
        }
    };
    std::mem::forget(database_box);

    result
}

#[no_mangle]
pub unsafe extern "C" fn database_set_head(database: *mut SyncDatabase<DirectoryLabelStore, DirectoryLayerStore>, layer_ptr: *const SyncDatabaseLayer<DirectoryLayerStore>, err: *mut *const c_char) -> bool {
    match (*database).set_head(&*layer_ptr) {
        Ok(b) => {
            *err = std::ptr::null();
            b

        },
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn store_create_base_layer(store_ptr: *mut SyncStore<DirectoryLabelStore, DirectoryLayerStore>, err: *mut *const c_char) -> *const SyncDatabaseLayerBuilder<DirectoryLayerStore> {
    let store = Box::from_raw(store_ptr);
    let result = match store.create_base_layer() {
        Ok(builder) => {
            *err = std::ptr::null();
            Box::into_raw(Box::new(builder))
        },
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            std::ptr::null()
        }
    };
    std::mem::forget(store);

    result
}

#[no_mangle]
pub unsafe extern "C" fn layer_open_write(layer_ptr: *mut SyncDatabaseLayer<DirectoryLayerStore>, err: *mut *const c_char) -> *const SyncDatabaseLayerBuilder<DirectoryLayerStore> {
    let layer = Box::from_raw(layer_ptr);
    let result = match layer.open_write() {
        Ok(builder) => {
            *err = std::ptr::null();
            Box::into_raw(Box::new(builder))
        },
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            std::ptr::null()
        }
    };

    std::mem::forget(layer);

    result
}

#[no_mangle]
pub unsafe extern "C" fn builder_add_id_triple(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, subject: u64, predicate: u64, object: u64, err: *mut *const c_char) -> bool {
    let builder = Box::from_raw(builder_ptr);

    let result = match builder.add_id_triple(IdTriple::new(subject, predicate, object)) {
        Ok(r) => {
            *err = std::ptr::null();

            r
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();

            false
        }
    };

    std::mem::forget(builder);

    result
}

#[no_mangle]
pub unsafe extern "C" fn builder_add_string_node_triple(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, subject_ptr: *const c_char, predicate_ptr: *const c_char, object_ptr: *const c_char, err: *mut *const c_char) {
    let builder = Box::from_raw(builder_ptr);

    let subject = CStr::from_ptr(subject_ptr).to_string_lossy();
    let predicate = CStr::from_ptr(predicate_ptr).to_string_lossy();
    let object = CStr::from_ptr(object_ptr).to_string_lossy();

    match builder.add_string_triple(&StringTriple::new_node(&subject, &predicate, &object)) {
        Ok(_) => *err = std::ptr::null(),
        Err(e) => *err = error_to_cstring(e).into_raw()
    };

    std::mem::forget(builder);
}

#[no_mangle]
pub unsafe extern "C" fn builder_add_string_value_triple(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, subject_ptr: *const c_char, predicate_ptr: *const c_char, object_ptr: *const c_char, err: *mut *const c_char) {
    let builder = Box::from_raw(builder_ptr);

    let subject = CStr::from_ptr(subject_ptr).to_string_lossy();
    let predicate = CStr::from_ptr(predicate_ptr).to_string_lossy();
    let object = CStr::from_ptr(object_ptr).to_string_lossy();

    match builder.add_string_triple(&StringTriple::new_value(&subject, &predicate, &object)) {
        Ok(_) => *err = std::ptr::null(),
        Err(e) => *err = error_to_cstring(e).into_raw()
    };

    std::mem::forget(builder);
}


#[no_mangle]
pub unsafe extern "C" fn builder_remove_id_triple(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, subject: u64, predicate: u64, object: u64, err: *mut *const c_char) -> bool {
    let builder = Box::from_raw(builder_ptr);

    let result = match builder.remove_id_triple(IdTriple::new(subject, predicate, object)) {
        Ok(r) => {
            *err = std::ptr::null();

            r
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();

            false
        }
    };

    std::mem::forget(builder);

    result
}

#[no_mangle]
pub unsafe extern "C" fn builder_remove_string_node_triple(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, subject_ptr: *const c_char, predicate_ptr: *const c_char, object_ptr: *const c_char, err: *mut *const c_char) -> bool {
    let builder = Box::from_raw(builder_ptr);

    let subject = CStr::from_ptr(subject_ptr).to_string_lossy();
    let predicate = CStr::from_ptr(predicate_ptr).to_string_lossy();
    let object = CStr::from_ptr(object_ptr).to_string_lossy();

    let result = match builder.remove_string_triple(&StringTriple::new_node(&subject, &predicate, &object)) {
        Ok(r) => {
            *err = std::ptr::null();

            r
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();

            false
        }
    };

    std::mem::forget(builder);

    result
}

#[no_mangle]
pub unsafe extern "C" fn builder_remove_string_value_triple(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, subject_ptr: *const c_char, predicate_ptr: *const c_char, object_ptr: *const c_char, err: *mut *const c_char) -> bool {
    let builder = Box::from_raw(builder_ptr);

    let subject = CStr::from_ptr(subject_ptr).to_string_lossy();
    let predicate = CStr::from_ptr(predicate_ptr).to_string_lossy();
    let object = CStr::from_ptr(object_ptr).to_string_lossy();

    let result = match builder.remove_string_triple(&StringTriple::new_value(&subject, &predicate, &object)) {
        Ok(r) => {
            *err = std::ptr::null();

            r
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();

            false
        }
    };

    std::mem::forget(builder);

    result
}

#[no_mangle]
pub unsafe extern "C" fn builder_commit(builder_ptr: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>, err: *mut *const c_char) -> *const SyncDatabaseLayer<DirectoryLayerStore> {
    let builder = Box::from_raw(builder_ptr);

    let result = match builder.commit() {
        Ok(layer) => {
            *err = std::ptr::null();
            Box::into_raw(Box::new(layer))
        }
        Err(e) => {
            *err = error_to_cstring(e).into_raw();
            std::ptr::null()
        }
    };

    std::mem::forget(builder);

    result
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_directory_store(store: *mut SyncStore<DirectoryLabelStore, DirectoryLayerStore>) {
    Box::from_raw(store);
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_db(db: *mut SyncDatabase<DirectoryLabelStore, DirectoryLayerStore>) {
    Box::from_raw(db);
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_layer(layer: *mut SyncDatabaseLayer<DirectoryLayerStore>) {
    Box::from_raw(layer);
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_layer_builder(layer_builder: *mut SyncDatabaseLayerBuilder<DirectoryLayerStore>) {
    Box::from_raw(layer_builder);
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_cstring(cstring_ptr: *mut c_char) {
    CString::from_raw(cstring_ptr);
}
