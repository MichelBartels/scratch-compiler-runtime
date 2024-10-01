use std::{
    thread,
    time::{Duration, Instant},
};

use super::ui::*;

#[no_mangle]
pub fn motion_add_costume(sprite: *const WrappedSprite, costume: *mut Costume) {
    let sprite = unsafe { &*sprite };
    let costume = unsafe { Box::from_raw(costume) };
    sprite.write().unwrap().costumes.push(*costume)
}

#[no_mangle]
pub fn motion_set_x(sprite: *const WrappedSprite, x: f64) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (_, y) = sprite.position.get_position();
    sprite.position = Position::Constant(x as f32, y);
}

#[no_mangle]
pub fn motion_set_y(sprite: *const WrappedSprite, y: f64) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (x, _) = sprite.position.get_position();
    sprite.position = Position::Constant(x, y as f32);
}

#[no_mangle]
pub fn motion_change_x(sprite: *const WrappedSprite, dx: f64) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (x, y) = sprite.position.get_position();
    sprite.position = Position::Constant(x + dx as f32, y);
}

#[no_mangle]
pub fn motion_change_y(sprite: *const WrappedSprite, dy: f64) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (x, y) = sprite.position.get_position();
    sprite.position = Position::Constant(x, y + dy as f32);
}

#[no_mangle]
pub fn motion_get_x(sprite: *const WrappedSprite) -> f64 {
    let sprite = unsafe { &*sprite };
    let pos = sprite.read().unwrap().position.get_position().0 as f64;
    pos
}

#[no_mangle]
pub fn motion_get_y(sprite: *const WrappedSprite) -> f64 {
    let sprite = unsafe { &*sprite };
    sprite.read().unwrap().position.get_position().1 as f64
}

#[no_mangle]
pub fn motion_get_direction(sprite: *const WrappedSprite) -> f64 {
    let sprite = unsafe { &*sprite };
    sprite.read().unwrap().direction as f64
}

#[no_mangle]
pub fn motion_turn_right(sprite: *const WrappedSprite, degrees: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().direction += degrees as f32;
}

#[no_mangle]
pub fn motion_turn_left(sprite: *const WrappedSprite, degrees: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().direction -= degrees as f32;
}

#[no_mangle]
pub fn motion_move_steps(sprite: *const WrappedSprite, steps: f64) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let direction = sprite.direction.to_radians();
    let (x, y) = sprite.position.get_position();
    sprite.position = Position::Constant(
        x - steps as f32 * direction.cos(),
        y - steps as f32 * direction.sin(),
    );
}

#[no_mangle]
pub fn motion_glide_to_xy(sprite: *const WrappedSprite, x: f64, y: f64, duration: f64) {
    let duration = Duration::from_secs_f64(duration);
    {
        let sprite = unsafe { &*sprite };
        let mut sprite = sprite.write().unwrap();
        let (start_x, start_y) = sprite.position.get_position();
        sprite.position = Position::Glide {
            start_x,
            start_y,
            end_x: x as f32,
            end_y: y as f32,
            duration,
            start_time: Instant::now(),
        };
    };
    thread::sleep(duration); // switch to sleep_until when stable
}

#[no_mangle]
pub fn motion_glide_to_sprite(
    sprite: *const WrappedSprite,
    target: *const WrappedSprite,
    duration: f64,
) {
    let (target_x, target_y) = {
        let target = unsafe { &*target };
        let target = target.read().unwrap();
        target.position.get_position()
    };
    motion_glide_to_xy(sprite, target_x as f64, target_y as f64, duration);
}

#[no_mangle]
pub fn motion_glide_to_cursor(sprite: *const WrappedSprite, scene: *const Scene, duration: f64) {
    let (x, y) = {
        let cursor = unsafe { &*scene }.cursor.read().unwrap();
        (cursor.0, cursor.1)
    };
    motion_glide_to_xy(sprite, x as f64, y as f64, duration);
}

fn random_position() -> (f32, f32) {
    (
        rand::random::<f32>() * 480.0 - 240.0,
        rand::random::<f32>() * 360.0 - 180.0,
    )
}

#[no_mangle]
pub fn motion_glide_to_random_position(sprite: *const WrappedSprite, duration: f64) {
    let (x, y) = random_position();
    motion_glide_to_xy(sprite, x as f64, y as f64, duration);
}

#[no_mangle]
pub fn motion_point_towards_sprite(sprite: *const WrappedSprite, target: *const WrappedSprite) {
    let sprite = unsafe { &*sprite };
    let target = unsafe { &*target };
    let mut sprite = sprite.write().unwrap();
    let target = target.read().unwrap();
    let (target_x, target_y) = target.position.get_position();
    sprite.point_towards(target_x, target_y);
}

#[no_mangle]
pub fn motion_point_towards_cursor(sprite: *const WrappedSprite, scene: *const Scene) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let cursor = unsafe { &*scene }.cursor.read().unwrap();
    sprite.point_towards(cursor.0, 180.0);
}

#[no_mangle]
pub fn motion_point_towards_random_position(sprite: *const WrappedSprite) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (x, y) = random_position();
    sprite.point_towards(x, y);
}

#[no_mangle]
pub fn motion_go_to_random_position(sprite: *const WrappedSprite) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (x, y) = random_position();
    sprite.position = Position::Constant(x, y);
}

#[no_mangle]
pub fn motion_go_to_sprite(sprite: *const WrappedSprite, target: *const WrappedSprite) {
    let sprite = unsafe { &*sprite };
    let target = unsafe { &*target };
    let mut sprite = sprite.write().unwrap();
    let target = target.read().unwrap();
    let (target_x, target_y) = target.position.get_position();
    sprite.position = Position::Constant(target_x, target_y);
}

#[no_mangle]
pub fn motion_go_to_cursor(sprite: *const WrappedSprite, scene: *const Scene) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let cursor = unsafe { &*scene }.cursor.read().unwrap();
    sprite.position = Position::Constant(cursor.0 - 240.0, 180.0 - cursor.1);
}

#[no_mangle]
pub fn motion_if_on_edge_bounce(sprite: *const WrappedSprite) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let (x, y) = sprite.position.get_position();
    let (rotation_center_x, rotation_center_y) = {
        let costume = &sprite.costumes[sprite.current_costume];
        (costume.rotation_center_x, costume.rotation_center_y)
    };
    if x.abs() >= 240.0 - rotation_center_x {
        sprite.direction = -norm_angle(sprite.direction).copysign(x);
    };
    if y.abs() >= 180.0 - rotation_center_y {
        sprite.direction = norm_angle(sprite.direction - 90.0).copysign(y) + 90.0;
    };
    sprite.position = Position::Constant(
        x.clamp(-240.0 + rotation_center_x, 240.0 - rotation_center_x),
        y.clamp(-180.0 + rotation_center_y, 180.0 - rotation_center_y),
    );
}

#[no_mangle]
pub fn motion_set_rotation_style(sprite: *const WrappedSprite, rotation_style: i32) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().rotation_style = RotationStyle::from_i32(rotation_style);
}
