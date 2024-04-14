use std::ffi::{c_void, CStr};
use std::io::{self, BufRead, Write};

#[no_mangle]
pub extern "C" fn alloc_string(c_str: *const i8) -> *mut c_void {
    let c_str = unsafe { CStr::from_ptr(c_str) };
    let str = c_str.to_str().unwrap().to_owned();
    let boxed_str = Box::new(str);
    Box::into_raw(boxed_str) as *mut c_void
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_void) {
    unsafe {
        let _ = Box::from_raw(ptr as *mut String);
    }
}

#[no_mangle]
pub extern "C" fn say(ptr: *const c_void) {
    let s = unsafe { &*(ptr as *const String) };
    println!("Saying: {}", s);
}

#[no_mangle]
pub extern "C" fn ask(question: *const c_void) -> *mut c_void {
    let question = unsafe { &*(question as *const String) };
    print!("Asking: {} ", question);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap();
    let input = input.trim().to_owned();
    let boxed_str = Box::new(input);
    Box::into_raw(boxed_str) as *mut c_void
}

fn alloc_empty_vec<T>() -> *mut c_void {
    let vec: Vec<T> = Vec::new();
    let boxed_vec = Box::new(vec);
    Box::into_raw(boxed_vec) as *mut c_void
}

#[no_mangle]
pub extern "C" fn alloc_empty_string_vec() -> *mut c_void {
    alloc_empty_vec::<String>()
}

#[no_mangle]
pub extern "C" fn alloc_empty_f64_vec() -> *mut c_void {
    alloc_empty_vec::<f64>()
}

#[no_mangle]
pub extern "C" fn alloc_empty_bool_vec() -> *mut c_void {
    alloc_empty_vec::<bool>()
}

fn free_vec<T>(ptr: *mut c_void) {
    unsafe {
        let _ = Box::from_raw(ptr as *mut Vec<T>);
    }
}

#[no_mangle]
pub extern "C" fn free_string_vec(ptr: *mut c_void) {
    free_vec::<String>(ptr);
}

#[no_mangle]
pub extern "C" fn free_f64_vec(ptr: *mut c_void) {
    free_vec::<f64>(ptr);
}

#[no_mangle]
pub extern "C" fn free_bool_vec(ptr: *mut c_void) {
    free_vec::<bool>(ptr);
}

fn push_to_vec<T>(ptr: *mut c_void, value: T) {
    let vec = unsafe { &mut *(ptr as *mut Vec<T>) };
    vec.push(value);
}

#[no_mangle]
pub extern "C" fn push_to_string_vec(ptr: *mut c_void, value: *const c_void) {
    let value = unsafe { &*(value as *const String) };
    push_to_vec(ptr, value.clone());
}

#[no_mangle]
pub extern "C" fn push_to_f64_vec(ptr: *mut c_void, value: f64) {
    push_to_vec(ptr, value);
}

#[no_mangle]
pub extern "C" fn push_to_bool_vec(ptr: *mut c_void, value: bool) {
    push_to_vec(ptr, value);
}
