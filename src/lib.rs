use std::ffi::{c_char, c_void, CStr};
use std::fmt::Debug;
use std::io::{self, BufRead, Write};
use std::sync::RwLock;
use std::thread::JoinHandle;

mod ui;
pub use ui::{create_window, new_scene, new_sprite, scene_add_sprite};

#[no_mangle]
pub extern "C" fn alloc_string(c_str: *const c_char) -> *mut String {
    let c_str = unsafe { CStr::from_ptr(c_str) };
    let str = c_str.to_str().unwrap().to_owned();
    let boxed_str = Box::new(str);
    Box::into_raw(boxed_str)
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut String) {
    unsafe {
        let _ = Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn say(ptr: *const String) {
    let s = unsafe { &*ptr };
    println!("{}", s);
}

#[no_mangle]
pub extern "C" fn ask(question: *const String) -> *mut c_void {
    let question = unsafe { &*question };
    print!("{} ", question);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap();
    let input = input.trim().to_owned();
    let boxed_input = Box::new(input);
    Box::into_raw(boxed_input) as *mut c_void
}

fn alloc_empty_vec<T>() -> *mut RwLock<Vec<T>> {
    let vec: Vec<T> = Vec::new();
    let boxed_vec = Box::new(RwLock::new(vec));
    Box::into_raw(boxed_vec)
}

#[no_mangle]
pub extern "C" fn alloc_empty_string_vec() -> *mut RwLock<Vec<String>> {
    alloc_empty_vec::<String>()
}

#[no_mangle]
pub extern "C" fn alloc_empty_f64_vec() -> *mut RwLock<Vec<f64>> {
    alloc_empty_vec::<f64>()
}

#[no_mangle]
pub extern "C" fn alloc_empty_bool_vec() -> *mut RwLock<Vec<bool>> {
    alloc_empty_vec::<bool>()
}

fn clear_vec<T: std::fmt::Debug>(ptr: *mut RwLock<Vec<T>>) {
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let mut vec = rwlock.write().unwrap();
    vec.clear();
}

#[no_mangle]
pub extern "C" fn clear_string_vec(ptr: *mut RwLock<Vec<String>>) {
    clear_vec::<String>(ptr);
}

#[no_mangle]
pub extern "C" fn clear_f64_vec(ptr: *mut RwLock<Vec<f64>>) {
    clear_vec::<f64>(ptr);
}

#[no_mangle]
pub extern "C" fn clear_bool_vec(ptr: *mut RwLock<Vec<bool>>) {
    clear_vec::<bool>(ptr);
}

fn push_to_vec<T: Debug>(ptr: *mut RwLock<Vec<T>>, value: T) {
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let mut vec = rwlock.write().unwrap();
    vec.push(value);
}

#[no_mangle]
pub extern "C" fn push_to_string_vec(ptr: *mut RwLock<Vec<String>>, value: *const String) {
    let value = unsafe { &*value };
    push_to_vec(ptr, value.clone());
}

#[no_mangle]
pub extern "C" fn push_to_f64_vec(ptr: *mut RwLock<Vec<f64>>, value: f64) {
    push_to_vec(ptr, value);
}

#[no_mangle]
pub extern "C" fn push_to_bool_vec(ptr: *mut RwLock<Vec<bool>>, value: bool) {
    push_to_vec(ptr, value);
}

fn get_vec_element<T: Clone + Debug>(ptr: *const RwLock<Vec<T>>, index: f64, default: T) -> T {
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let vec = rwlock.read().unwrap();
    let index = index as usize - 1;
    vec.get(index).map(|x| x.clone()).unwrap_or(default)
}

#[no_mangle]
pub extern "C" fn get_string_vec_element(
    ptr: *const RwLock<Vec<String>>,
    index: f64,
) -> *mut String {
    let element = get_vec_element(ptr, index, "".to_owned());
    let boxed_element = Box::new(element);
    Box::into_raw(boxed_element)
}

#[no_mangle]
pub extern "C" fn get_f64_vec_element(ptr: *const RwLock<Vec<f64>>, index: f64) -> f64 {
    get_vec_element(ptr, index, 0.0)
}

#[no_mangle]
pub extern "C" fn get_bool_vec_element(ptr: *const RwLock<Vec<bool>>, index: f64) -> bool {
    get_vec_element(ptr, index, false)
}

fn index_of<T: PartialEq + Debug>(ptr: *const RwLock<Vec<T>>, value: T) -> f64 {
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let vec = rwlock.read().unwrap();
    vec.iter()
        .position(|x| *x == value)
        .map(|i| i as f64 + 1.0)
        .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn index_of_string(vec: *const RwLock<Vec<String>>, value: *const String) -> f64 {
    let value = unsafe { &*value };
    let index = index_of(vec, value.clone());
    index
}

#[no_mangle]
pub extern "C" fn index_of_f64(vec: *const RwLock<Vec<f64>>, value: f64) -> f64 {
    index_of(vec, value)
}

#[no_mangle]
pub extern "C" fn index_of_bool(vec: *const RwLock<Vec<bool>>, value: bool) -> f64 {
    index_of(vec, value)
}

fn set_vec_element<T>(ptr: *const RwLock<Vec<T>>, index: f64, value: T) {
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let mut vec = rwlock.write().unwrap();
    let index = index as usize - 1;
    if index < vec.len() {
        vec[index] = value;
    }
}

#[no_mangle]
pub extern "C" fn set_string_vec_element(
    ptr: *const RwLock<Vec<String>>,
    index: f64,
    value: *const String,
) {
    let value = unsafe { &*value };
    set_vec_element(ptr, index, value.clone());
}

#[no_mangle]
pub extern "C" fn set_f64_vec_element(ptr: *const RwLock<Vec<f64>>, index: f64, value: f64) {
    set_vec_element(ptr, index, value);
}

#[no_mangle]
pub extern "C" fn set_bool_vec_element(ptr: *const RwLock<Vec<bool>>, index: f64, value: bool) {
    set_vec_element(ptr, index, value);
}

fn len_of_vec<T: Debug>(ptr: *const RwLock<Vec<T>>) -> f64 {
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let vec = rwlock.read().unwrap();
    vec.len() as f64
}

#[no_mangle]
pub extern "C" fn len_of_string_vec(ptr: *const RwLock<Vec<String>>) -> f64 {
    len_of_vec::<String>(ptr)
}

#[no_mangle]
pub extern "C" fn len_of_f64_vec(ptr: *const RwLock<Vec<f64>>) -> f64 {
    len_of_vec::<f64>(ptr)
}

#[no_mangle]
pub extern "C" fn len_of_bool_vec(ptr: *const RwLock<Vec<bool>>) -> f64 {
    len_of_vec::<bool>(ptr)
}

fn cast_vec_to_string<T: ToString>(vec: *const RwLock<Vec<T>>) -> *mut String {
    let vec = unsafe { &*(vec as *const Vec<T>) };
    let string = vec
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let boxed_str = Box::new(string);
    Box::into_raw(boxed_str)
}

#[no_mangle]
pub extern "C" fn cast_string_vec_to_string(vec: *const RwLock<Vec<String>>) -> *mut String {
    cast_vec_to_string::<String>(vec)
}

#[no_mangle]
pub extern "C" fn cast_f64_vec_to_string(vec: *const RwLock<Vec<f64>>) -> *mut String {
    cast_vec_to_string::<f64>(vec)
}

fn cast_to_string<T: ToString>(val: T) -> *mut String {
    let string = val.to_string();
    let boxed_str = Box::new(string);
    Box::into_raw(boxed_str)
}

#[no_mangle]
pub extern "C" fn cast_f64_to_string(value: f64) -> *mut String {
    cast_to_string::<f64>(value)
}

#[no_mangle]
pub extern "C" fn cast_string_to_f64(value: *const String) -> f64 {
    let value = unsafe { &*(value) };
    value.parse().unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn join(string1: *const String, string2: *const String) -> *mut String {
    let string1 = unsafe { &*(string1 as *const String) };
    let string2 = unsafe { &*(string2 as *const String) };
    let joined = format!("{}{}", string1, string2);
    let boxed_str = Box::new(joined);
    Box::into_raw(boxed_str)
}

#[no_mangle]
pub extern "C" fn letter_of(string: *const String, index: f64) -> *mut String {
    let string = unsafe { &*(string) };
    let index = index as usize - 1;
    let letter = string
        .chars()
        .nth(index)
        .map(|c| c.to_string())
        .unwrap_or("".to_string());
    let boxed_str = Box::new(letter);
    Box::into_raw(boxed_str)
}

#[no_mangle]
pub extern "C" fn string_eq(string1: *const String, string2: *const String) -> bool {
    let string1 = unsafe { &*(string1) };
    let string2 = unsafe { &*(string2) };
    string1 == string2
}

#[no_mangle]
pub extern "C" fn spawn_thread(unsafe_fn: extern "C" fn()) -> *mut JoinHandle<()> {
    let handle = std::thread::spawn(move || {
        unsafe_fn();
    });
    let boxed_handle = Box::new(handle);
    Box::into_raw(boxed_handle)
}

#[no_mangle]
pub extern "C" fn join_thread(handle: *mut JoinHandle<()>) {
    let handle = unsafe { Box::from_raw(handle) };
    handle.join().unwrap();
}
