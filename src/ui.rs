use std::f32::consts::PI;
use std::ffi::{c_char, CStr};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use macroquad::math::Vec2;
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::{
    color, prelude::ImageFormat, texture::Texture2D, window::{clear_background, next_frame}, Window
};

fn svg_to_texture(svg_str: &str) -> Texture2D {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opt).unwrap();
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    let png = pixmap.encode_png().unwrap();
    Texture2D::from_file_with_format(&png, Some(ImageFormat::Png))
}

enum LazyTexture {
    Loaded(Texture2D),
    Unloaded(String),
}

impl LazyTexture {
    fn get_texture(&mut self) -> &Texture2D {
        match self {
            Self::Loaded(texture) => texture,
            Self::Unloaded(svg) => {
                let texture = svg_to_texture(svg);
                *self = Self::Loaded(texture);
                self.get_texture()
            }
        }
    }
}

fn norm_angle(angle: f32) -> f32 {
    (angle + 180.0).rem_euclid(360.0) - 180.0
}

pub struct Costume {
    svg: LazyTexture,
    rotation_center_x: f32,
    rotation_center_y: f32,
}

impl Costume {
    fn new(svg: String, rotation_center_x: i32, rotation_center_y: i32) -> Self {
        Self { svg: LazyTexture::Unloaded(svg), rotation_center_x: rotation_center_x as f32, rotation_center_y: rotation_center_y as f32 }
    }

    fn draw(&mut self, x: f32, y: f32, rotation: f32, rotation_style: RotationStyle) {
        let (rotation, flip_x) = match rotation_style {
            RotationStyle::AllAround => ((rotation - 90.) * PI / 180.0, false),
            RotationStyle::LeftRight => (0.0, match norm_angle(rotation) {
                r if r < 0.0 => true,
                _ => false,
            }),
            RotationStyle::DontRotate => (0.0, false),
        };
        draw_texture_ex(self.svg.get_texture(), x, y, color::WHITE, DrawTextureParams {
            rotation,
            pivot: Some(Vec2 {
                x: self.rotation_center_x + x,
                y: self.rotation_center_y + y,

            }),
            flip_x,
            ..Default::default()
        })
    }
}

#[no_mangle]
pub fn new_costume(svg_str: *const c_char, x: i32, y: i32) -> *const Costume {
    let svg_str = unsafe { CStr::from_ptr(svg_str).to_str().unwrap().to_owned() };
    let costume = Costume::new(svg_str, x, y);
    Box::into_raw(Box::new(costume))
}

enum Position {
    Constant(f32, f32),
    Glide {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        duration: Duration,
        start_time: Instant,
    },
}

impl Position {
    fn get_position(&self) -> (f32, f32) {
        match self {
            Self::Constant(x, y) => (*x, *y),
            Self::Glide {
                start_x,
                start_y,
                end_x,
                end_y,
                duration,
                start_time,
            } => {
                let elapsed = start_time.elapsed();
                let progress = elapsed.as_secs_f32() / duration.as_secs_f32();
                let progress = progress.min(1.0);
                let x = start_x + (end_x - start_x) * progress;
                let y = start_y + (end_y - start_y) * progress;
                (x, y)
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum RotationStyle {
    AllAround,
    LeftRight,
    DontRotate,
}

impl RotationStyle {
    fn from_i32(i: i32) -> Self {
        match i {
            0 => Self::AllAround,
            1 => Self::LeftRight,
            2 => Self::DontRotate,
            _ => panic!("Invalid rotation style"),
        }
    }
}

pub struct Sprite {
    costumes: Vec<Costume>,
    current_costume: usize,
    position: Position,
    direction: f32,
    rotation_style: RotationStyle,
}

impl Sprite {
    fn draw(&mut self) {
        let costume = &mut self.costumes[self.current_costume];
        let (x, y) = self.position.get_position();
        let x = 240. - costume.rotation_center_x + x;
        let y = 180. - costume.rotation_center_y - y;
        costume.draw(x, y, self.direction, self.rotation_style);
    }
    fn point_towards(&mut self, x: f32, y: f32) {
        let (current_x, current_y) = self.position.get_position();
        let dx = x - current_x;
        let dy = y - current_y;
        self.direction = 90.0 - dy.atan2(dx).to_degrees();
    }
}

pub type WrappedSprite = Arc<RwLock<Sprite>>;

#[no_mangle]
pub fn new_sprite(current_costume: i32, x: f32, y: f32, direction: f32, rotation_style: i32) -> *const WrappedSprite {
    let sprite = Sprite {
        costumes: Vec::new(),
        current_costume: current_costume as usize,
        position: Position::Constant(x, y),
        direction,
        rotation_style: RotationStyle::from_i32(rotation_style),
    };
    let arc = Arc::new(RwLock::new(sprite));
    Box::into_raw(Box::new(arc))
}

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
    sprite.position = Position::Constant(x - steps as f32 * direction.cos(), y - steps as f32 * direction.sin());
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
pub fn motion_glide_to_sprite(sprite: *const WrappedSprite, target: *const WrappedSprite, duration: f64) {
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
    (rand::random::<f32>() * 480.0 - 240.0, rand::random::<f32>() * 360.0 - 180.0)
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
    sprite.position = Position::Constant(x.clamp(-240.0 + rotation_center_x, 240.0 - rotation_center_x), y.clamp(-180.0 + rotation_center_y, 180.0 - rotation_center_y));
}

#[no_mangle]
pub fn motion_set_rotation_style(sprite: *const WrappedSprite, rotation_style: i32) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().rotation_style = RotationStyle::from_i32(rotation_style);
}

pub struct Scene {
    sprites: Vec<WrappedSprite>,
    cursor: RwLock<(f32, f32)>,
}

impl Scene {
    fn draw(&self) {
        for sprite in self.sprites.iter() {
            sprite.write().unwrap().draw();
        }
        *self.cursor.write().unwrap() = {
            let cursor = macroquad::input::mouse_position();
            (cursor.0 - 240.0, 180.0 - cursor.1)
        };
    }
}

#[no_mangle]
pub fn new_scene() -> *const Scene {
    Box::into_raw(Box::new(Scene { sprites: Vec::new(), cursor: RwLock::new((0., 0.))}))
}

#[no_mangle]
pub fn scene_add_sprite(scene: *mut Scene, sprite: *const WrappedSprite) {
    let scene = unsafe { &mut *scene };
    let sprite = unsafe { &*sprite };
    scene.sprites.push(sprite.clone());
}

#[no_mangle]
pub fn create_window(scene: *const Scene) {
    let scene = unsafe { &*scene };
    Window::from_config(macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "Scratch".to_owned(),
            window_width: 480,
            window_height: 360,
            high_dpi: true,
            ..Default::default()
        },
        ..Default::default()
    }, window_loop(scene));
}

async fn window_loop(scene: &Scene) {
    loop {
        clear_background(color::WHITE);
        scene.draw();
        next_frame().await
    }
}
