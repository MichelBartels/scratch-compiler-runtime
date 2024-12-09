use rand::Rng;

#[no_mangle]
pub extern "C" fn operator_random(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

#[no_mangle]
pub extern "C" fn operator_abs(value: f64) -> f64 {
    value.abs()
}

#[no_mangle]
pub extern "C" fn operator_floor(value: f64) -> f64 {
    value.floor()
}

#[no_mangle]
pub extern "C" fn operator_ceil(value: f64) -> f64 {
    value.ceil()
}

#[no_mangle]
pub extern "C" fn operator_sqrt(value: f64) -> f64 {
    value.sqrt()
}

#[no_mangle]
pub extern "C" fn operator_round(value: f64) -> f64 {
    value.round()
}

#[no_mangle]
pub extern "C" fn operator_contains(str: *const String, substr: *const String) -> bool {
    let str = unsafe { &*str };
    let substr = unsafe { &*substr };
    str.contains(substr)
}

#[no_mangle]
pub extern "C" fn operator_length(str: *const String) -> usize {
    let str = unsafe { &*str };
    str.len()
}
