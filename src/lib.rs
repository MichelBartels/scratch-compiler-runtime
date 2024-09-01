use std::ffi::{c_void, CStr, c_char};
use std::fmt::Debug;
use std::io::{self, BufRead, Write};
use std::sync::RwLock;

use macroquad::{
    color,
    window::{clear_background, next_frame},
    Window,
};

#[no_mangle]
pub extern "C" fn alloc_string(c_str: *const c_char) -> *mut c_void {
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
    println!("{}", s);
}

#[no_mangle]
pub extern "C" fn ask(question: *const c_void) -> *mut c_void {
    let question = unsafe { &*(question as *const String) };
    print!("{} ", question);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap();
    let input = input.trim().to_owned();
    let boxed_input = Box::new(input);
    Box::into_raw(boxed_input) as *mut c_void
}

fn alloc_empty_vec<T>() -> *mut c_void {
    let vec: Vec<T> = Vec::new();
    let boxed_vec = Box::new(RwLock::new(vec));
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

fn clear_vec<T: std::fmt::Debug>(ptr: *mut c_void) {
    let ptr = ptr.cast::<RwLock<Vec<T>>>();
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let mut vec = rwlock.write().unwrap();
    vec.clear();
}

#[no_mangle]
pub extern "C" fn clear_string_vec(ptr: *mut c_void) {
    clear_vec::<String>(ptr);
}

#[no_mangle]
pub extern "C" fn clear_f64_vec(ptr: *mut c_void) {
    clear_vec::<f64>(ptr);
}

#[no_mangle]
pub extern "C" fn clear_bool_vec(ptr: *mut c_void) {
    clear_vec::<bool>(ptr);
}

fn push_to_vec<T: Debug>(ptr: *mut c_void, value: T) {
    let ptr = ptr.cast::<RwLock<Vec<T>>>();
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let mut vec = rwlock.write().unwrap();
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

fn get_vec_element<T: Clone + Debug>(ptr: *mut c_void, index: f64, default: T) -> T {
    let ptr = ptr.cast::<RwLock<Vec<T>>>();
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let vec = rwlock.read().unwrap();
    let index = index as usize - 1;
    vec.get(index).map(|x| x.clone()).unwrap_or(default)
}

#[no_mangle]
pub extern "C" fn get_string_vec_element(ptr: *mut c_void, index: f64) -> *mut c_void {
    let element = get_vec_element(ptr, index, "".to_owned());
    let boxed_element = Box::new(element);
    Box::into_raw(boxed_element) as *mut c_void
}

#[no_mangle]
pub extern "C" fn get_f64_vec_element(ptr: *mut c_void, index: f64) -> f64 {
    get_vec_element(ptr, index, 0.0)
}

#[no_mangle]
pub extern "C" fn get_bool_vec_element(ptr: *mut c_void, index: f64) -> bool {
    get_vec_element(ptr, index, false)
}

fn index_of<T: PartialEq + Debug>(ptr: *mut c_void, value: T) -> f64 {
    let ptr = ptr.cast::<RwLock<Vec<T>>>();
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let vec = rwlock.read().unwrap();
    vec.iter()
        .position(|x| *x == value)
        .map(|i| i as f64 + 1.0)
        .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn index_of_string(vec: *mut c_void, value: *const c_void) -> f64 {
    let value = unsafe { &*(value as *const String) };
    let index = index_of(vec, value.clone());
    index
}

#[no_mangle]
pub extern "C" fn index_of_f64(vec: *mut c_void, value: f64) -> f64 {
    index_of(vec, value)
}

#[no_mangle]
pub extern "C" fn index_of_bool(vec: *mut c_void, value: bool) -> f64 {
    index_of(vec, value)
}

fn set_vec_element<T>(ptr: *mut c_void, index: f64, value: T) {
    let ptr = ptr.cast::<RwLock<Vec<T>>>();
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let mut vec = rwlock.write().unwrap();
    let index = index as usize - 1;
    if index < vec.len() {
        vec[index] = value;
    }
}

#[no_mangle]
pub extern "C" fn set_string_vec_element(ptr: *mut c_void, index: f64, value: *const c_void) {
    let value = unsafe { &*(value as *const String) };
    set_vec_element(ptr, index, value.clone());
}

#[no_mangle]
pub extern "C" fn set_f64_vec_element(ptr: *mut c_void, index: f64, value: f64) {
    set_vec_element(ptr, index, value);
}

#[no_mangle]
pub extern "C" fn set_bool_vec_element(ptr: *mut c_void, index: f64, value: bool) {
    set_vec_element(ptr, index, value);
}

fn len_of_vec<T: Debug>(ptr: *mut c_void) -> f64 {
    let ptr = ptr.cast::<RwLock<Vec<T>>>();
    let rwlock = unsafe { ptr.as_ref().unwrap() };
    let vec = rwlock.read().unwrap();
    vec.len() as f64
}

#[no_mangle]
pub extern "C" fn len_of_string_vec(ptr: *mut c_void) -> f64 {
    len_of_vec::<String>(ptr)
}

#[no_mangle]
pub extern "C" fn len_of_f64_vec(ptr: *mut c_void) -> f64 {
    len_of_vec::<f64>(ptr)
}

#[no_mangle]
pub extern "C" fn len_of_bool_vec(ptr: *mut c_void) -> f64 {
    len_of_vec::<bool>(ptr)
}

fn cast_vec_to_string<T: ToString>(vec: *mut c_void) -> *mut c_void {
    let vec = unsafe { &*(vec as *const Vec<T>) };
    let string = vec
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");
    let boxed_str = Box::new(string);
    Box::into_raw(boxed_str) as *mut c_void
}

#[no_mangle]
pub extern "C" fn cast_string_vec_to_string(vec: *mut c_void) -> *mut c_void {
    cast_vec_to_string::<String>(vec)
}

#[no_mangle]
pub extern "C" fn cast_f64_vec_to_string(vec: *mut c_void) -> *mut c_void {
    cast_vec_to_string::<f64>(vec)
}

fn cast_to_string<T: ToString>(val: T) -> *mut c_void {
    let string = val.to_string();
    let boxed_str = Box::new(string);
    Box::into_raw(boxed_str) as *mut c_void
}

#[no_mangle]
pub extern "C" fn cast_f64_to_string(value: f64) -> *mut c_void {
    cast_to_string::<f64>(value)
}

#[no_mangle]
pub extern "C" fn cast_string_to_f64(value: *const c_void) -> f64 {
    let value = unsafe { &*(value as *const String) };
    value.parse().unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn join(string1: *const c_void, string2: *const c_void) -> *mut c_void {
    let string1 = unsafe { &*(string1 as *const String) };
    let string2 = unsafe { &*(string2 as *const String) };
    let joined = format!("{}{}", string1, string2);
    let boxed_str = Box::new(joined);
    Box::into_raw(boxed_str) as *mut c_void
}

#[no_mangle]
pub extern "C" fn letter_of(string: *const c_void, index: f64) -> *mut c_void {
    let string = unsafe { &*(string as *const String) };
    let index = index as usize - 1;
    let letter = string
        .chars()
        .nth(index)
        .map(|c| c.to_string())
        .unwrap_or("".to_string());
    let boxed_str = Box::new(letter);
    Box::into_raw(boxed_str) as *mut c_void
}

#[no_mangle]
pub extern "C" fn string_eq(string1: *const c_void, string2: *const c_void) -> bool {
    let string1 = unsafe { &*(string1 as *const String) };
    let string2 = unsafe { &*(string2 as *const String) };
    string1 == string2
}

#[no_mangle]
pub extern "C" fn spawn_thread(unsafe_fn: extern "C" fn()) -> *mut c_void {
    let handle = std::thread::spawn(move || {
        unsafe_fn();
    });
    let boxed_handle = Box::new(handle);
    Box::into_raw(boxed_handle) as *mut c_void
}

#[no_mangle]
pub extern "C" fn join_thread(handle: *mut c_void) {
    let handle = unsafe { Box::from_raw(handle as *mut std::thread::JoinHandle<()>) };
    handle.join().unwrap();
}

#[no_mangle]
fn create_window() {
    Window::from_config(macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "Scratch".to_owned(),
            window_width: 480,
            window_height: 360,
            high_dpi: true,
            ..Default::default()
        },
        ..Default::default()
    }, window_loop());
}
async fn window_loop() {
    loop {
        clear_background(color::WHITE);
        next_frame().await
    }
}
