use crate::ui::{WrappedScene, WrappedSprite};

#[no_mangle]
pub extern "C" fn sensing_touches_cursor(
    sprite: *const WrappedSprite,
    scene: *const WrappedScene,
) -> bool {
    let sprite = unsafe { &*sprite };
    let scene = unsafe { &*scene };
    let (x, y) = scene.read().unwrap().cursor;
    sprite.read().unwrap().contains(x, y)
}

#[no_mangle]
pub extern "C" fn sensing_mousex(scene: *const WrappedScene) -> f64 {
    let scene = unsafe { &*scene };
    scene.read().unwrap().cursor.0 as f64
}

#[no_mangle]
pub extern "C" fn sensing_mousey(scene: *const WrappedScene) -> f64 {
    let scene = unsafe { &*scene };
    scene.read().unwrap().cursor.1 as f64
}
