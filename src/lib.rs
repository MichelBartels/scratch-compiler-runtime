use std::ffi::{c_void, CStr};
use std::fmt::Debug;
use std::io::{self, BufRead, Write};

#[no_mangle]
pub extern "C" fn alloc_string(c_str: *const u8) -> *mut c_void {
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
pub extern "C" fn ask(question: *const c_void, answer: *mut c_void) {
    let question = unsafe { &*(question as *const String) };
    print!("{} ", question);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap();
    println!("Input was: {}", input);
    let input = input.trim().to_owned();
    println!("Now owned");
    unsafe {
        *(answer as *mut String) = input;
    }
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

fn clear_vec<T: std::fmt::Debug>(ptr: *mut c_void) {
    let vec = unsafe { &mut *(ptr as *mut Vec<T>) };
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
    let ptr = ptr.cast::<Vec<T>>();
    let vec = unsafe { ptr.as_mut().expect("Failed to get mut reference") };
    println!("Pushing to vec: {:?}", vec);
    println!("Vec capacity: {:?}", vec.capacity());
    vec.push(value);
    println!("New vec capacity: {:?}", vec.capacity());
    println!("New vec: {:?}", vec);
}

#[no_mangle]
pub extern "C" fn push_to_string_vec(ptr: *mut c_void, value: *const c_void) {
    println!("vec address: {:p}", ptr);
    let value = unsafe { &*(value as *const String) };
    println!("Pushing to vec: {:?}", value);
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

#[no_mangle]
pub extern "C" fn get_string_vec_element(ptr: *mut c_void, index: f64) -> *mut c_void {
    let vec = unsafe { &*(ptr as *const Vec<String>) };
    let index = index as usize - 1;
    let element = vec.get(index).map(|str| str.to_string()).unwrap_or("".to_string());
    let boxed_str = Box::new(element.clone());
    Box::into_raw(boxed_str) as *mut c_void
}

#[no_mangle]
pub extern "C" fn get_f64_vec_element(ptr: *mut c_void, index: f64) -> f64 {
    let vec = unsafe { &*(ptr as *const Vec<f64>) };
    let index = index as usize - 1;
    *vec.get(index).unwrap_or(&0.0)
}

#[no_mangle]
pub extern "C" fn get_bool_vec_element(ptr: *mut c_void, index: f64) -> bool {
    let vec = unsafe { &*(ptr as *const Vec<bool>) };
    let index = index as usize - 1;
    *vec.get(index).unwrap_or(&false)
}

fn index_of<T: PartialEq + Debug>(vec: *mut c_void, value: T) -> f64 {
    let vec = unsafe { &*(vec as *const Vec<T>) };
    println!("{:?}", vec);
    println!("got vec and going to get position");
    vec.iter().position(|x| *x == value).map(|i| i as f64 + 1.0).unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn index_of_string(vec: *mut c_void, value: *const c_void) -> f64 {
    println!("index_of_string");
    println!("address of vec: '{:p}'", vec);
    let value = unsafe { &*(value as *const String) };
    println!("got value, {:?}", value);
    let index = index_of(vec, value.clone());
    println!("got index");
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
    let vec = unsafe { &mut *(ptr as *mut Vec<T>) };
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

fn len_of_vec<T>(ptr: *mut c_void) -> f64 {
    let vec = unsafe { &*(ptr as *const Vec<T>) };
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
    let string = vec.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ");
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
    let letter = string.chars().nth(index).map(|c| c.to_string()).unwrap_or("".to_string());
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
