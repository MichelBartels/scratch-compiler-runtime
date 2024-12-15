use crate::ui::WrappedSprite;

pub struct Broadcast(pub Vec<extern "C" fn()>);

#[no_mangle]
pub extern "C" fn new_broadcast() -> *mut Broadcast {
    Box::into_raw(Box::new(Broadcast(Vec::new())))
}

#[no_mangle]
pub extern "C" fn broadcast_add(broadcast: *mut Broadcast, f: extern "C" fn()) {
    unsafe {
        (*broadcast).0.push(f);
    }
}

#[no_mangle]
pub extern "C" fn event_broadcast(broadcast: *const Broadcast) {
    let broadcast = unsafe { &(*broadcast).0 };
    broadcast.iter().for_each(|f| {
        std::thread::spawn(move || f());
    });
}

#[no_mangle]
pub extern "C" fn event_broadcastandwait(broadcast: *const Broadcast) {
    let broadcast = unsafe { &(*broadcast).0 };
    let handles = broadcast.iter().map(|f| std::thread::spawn(move || f()));
    handles.for_each(|h| h.join().unwrap());
}

#[no_mangle]
pub extern "C" fn event_whenthisspriteclicked(sprite: *const WrappedSprite, f: extern "C" fn()) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    sprite.on_click.push(f)
}
