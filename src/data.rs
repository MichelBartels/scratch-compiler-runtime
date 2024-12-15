use std::sync::RwLock;

fn data_deleteoflist<T>(vec: *mut RwLock<Vec<T>>, index: f64) {
    let vec = unsafe { &mut *vec };
    let mut vec = vec.write().unwrap();
    let index = index as usize;
    if index == 0 {
        return;
    }
    let index = index - 1;
    if index < vec.len() {
        vec.remove(index);
    }
}

#[no_mangle]
pub extern "C" fn data_deleteoflist_string(vec: *mut RwLock<Vec<String>>, index: f64) {
    data_deleteoflist(vec, index);
}

#[no_mangle]
pub extern "C" fn data_deleteoflist_f64(vec: *mut RwLock<Vec<f64>>, index: f64) {
    data_deleteoflist(vec, index);
}

#[no_mangle]
pub extern "C" fn data_deleteoflist_bool(vec: *mut RwLock<Vec<bool>>, index: f64) {
    data_deleteoflist(vec, index);
}

fn data_insertatlist<T>(vec: *mut RwLock<Vec<T>>, index: f64, value: T) {
    let vec = unsafe { &mut *vec };
    let mut vec = vec.write().unwrap();
    let index = index as usize;
    if index == 0 {
        return;
    }
    let index = index - 1;
    if index <= vec.len() {
        vec.insert(index, value);
    }
}

#[no_mangle]
pub extern "C" fn data_insertatlist_string(
    vec: *mut RwLock<Vec<String>>,
    index: f64,
    value: *mut String,
) {
    let value = unsafe { Box::from_raw(value) };
    data_insertatlist(vec, index, *value);
}

#[no_mangle]
pub extern "C" fn data_insertatlist_f64(vec: *mut RwLock<Vec<f64>>, index: f64, value: f64) {
    data_insertatlist(vec, index, value);
}

#[no_mangle]
pub extern "C" fn data_insertatlist_bool(vec: *mut RwLock<Vec<bool>>, index: f64, value: bool) {
    data_insertatlist(vec, index, value);
}
