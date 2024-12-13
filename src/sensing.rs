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
