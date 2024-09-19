use std::ffi::{c_char, CStr};
use std::sync::{Arc, RwLock};

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

pub struct Costume {
    svg: LazyTexture,
    rotation_center_x: f32,
    rotation_center_y: f32,
}

impl Costume {
    fn new(svg: String, rotation_center_x: i32, rotation_center_y: i32) -> Self {
        Self { svg: LazyTexture::Unloaded(svg), rotation_center_x: rotation_center_x as f32, rotation_center_y: rotation_center_y as f32 }
    }

    fn draw(&mut self, x: f32, y: f32, rotation: f32) {
        draw_texture_ex(self.svg.get_texture(), x, y, color::WHITE, DrawTextureParams {
            rotation: (rotation - 90.) * std::f32::consts::PI / 180.0,
            pivot: Some(Vec2 {
                x: self.rotation_center_x + x,
                y: self.rotation_center_y + y,

            }),
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

pub struct Sprite {
    costumes: Vec<Costume>,
    current_costume: usize,
    x: f32,
    y: f32,
    direction: f32,
}

impl Sprite {
    fn draw(&mut self) {
        let costume = &mut self.costumes[self.current_costume];
        let x = 240. - costume.rotation_center_x + self.x;
        let y = 180. - costume.rotation_center_y - self.y;
        costume.draw(x, y, self.direction);
    }
}

pub type WrappedSprite = Arc<RwLock<Sprite>>;

#[no_mangle]
pub fn new_sprite(current_costume: i32, x: f32, y: f32, direction: f32) -> *const WrappedSprite {
    let sprite = Sprite {
        costumes: Vec::new(),
        current_costume: current_costume as usize,
        x,
        y,
        direction,
    };
    let arc = Arc::new(RwLock::new(sprite));
    Box::into_raw(Box::new(arc))
}

#[no_mangle]
pub fn sprite_add_costume(sprite: *const WrappedSprite, costume: *mut Costume) {
    let sprite = unsafe { &*sprite };
    let costume = unsafe { Box::from_raw(costume) };
    sprite.write().unwrap().costumes.push(*costume)
}

#[no_mangle]
pub fn sprite_set_x(sprite: *const WrappedSprite, x: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().x = x as f32;
}

#[no_mangle]
pub fn sprite_set_y(sprite: *const WrappedSprite, y: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().y = y as f32;
}

#[no_mangle]
pub fn sprite_change_x(sprite: *const WrappedSprite, dx: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().x += dx as f32;
}

#[no_mangle]
pub fn sprite_change_y(sprite: *const WrappedSprite, dy: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().y += dy as f32;
}

#[no_mangle]
pub fn sprite_get_x(sprite: *const WrappedSprite) -> f64 {
    let sprite = unsafe { &*sprite };
    sprite.read().unwrap().x as f64
}

#[no_mangle]
pub fn sprite_get_y(sprite: *const WrappedSprite) -> f64 {
    let sprite = unsafe { &*sprite };
    sprite.read().unwrap().y as f64
}

#[no_mangle]
pub fn sprite_get_direction(sprite: *const WrappedSprite) -> f64 {
    let sprite = unsafe { &*sprite };
    sprite.read().unwrap().direction as f64
}

#[no_mangle]
pub fn sprite_turn_right(sprite: *const WrappedSprite, degrees: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().direction += degrees as f32;
}

#[no_mangle]
pub fn sprite_turn_left(sprite: *const WrappedSprite, degrees: f64) {
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().direction -= degrees as f32;
}

#[no_mangle]
pub fn sprite_move_steps(sprite: *const WrappedSprite, steps: f64) {
    let sprite = unsafe { &*sprite };
    let mut sprite = sprite.write().unwrap();
    let direction = sprite.direction.to_radians();
    sprite.x -= steps as f32 * direction.cos();
    sprite.y -= steps as f32 * direction.sin();
}

pub struct Scene {
    sprites: Vec<WrappedSprite>,
}

impl Scene {
    fn draw(&self) {
        for sprite in self.sprites.iter() {
            sprite.write().unwrap().draw();
        }
    }
}

#[no_mangle]
pub fn new_scene() -> *const Scene {
    Box::into_raw(Box::new(Scene { sprites: Vec::new()}))
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
