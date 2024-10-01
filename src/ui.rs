use std::f32::consts::PI;
use std::ffi::{c_char, CStr};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use macroquad::math::Vec2;
use macroquad::text::{load_ttf_font, Font};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::{
    color, prelude::ImageFormat, texture::Texture2D, window::{clear_background, next_frame}, Window
};

use crate::looks::{Boundary, Bubble};

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

pub fn norm_angle(angle: f32) -> f32 {
    (angle + 180.0).rem_euclid(360.0) - 180.0
}

pub struct Costume {
    pub svg: LazyTexture,
    pub rotation_center_x: f32,
    pub rotation_center_y: f32,
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

pub enum Position {
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
    pub fn get_position(&self) -> (f32, f32) {
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
    pub fn from_i32(i: i32) -> Self {
        match i {
            0 => Self::AllAround,
            1 => Self::LeftRight,
            2 => Self::DontRotate,
            _ => panic!("Invalid rotation style"),
        }
    }
}

pub struct Sprite {
    pub costumes: Vec<Costume>,
    pub current_costume: usize,
    pub position: Position,
    pub direction: f32,
    pub rotation_style: RotationStyle,
    pub bubble: Option<Bubble>,
}

impl Sprite {
    fn costume(&mut self) -> &mut Costume {
        &mut self.costumes[self.current_costume]
    }
    fn draw(&mut self, font: &Font) {
        let costume = &mut self.costumes[self.current_costume];
        let (x, y) = self.position.get_position();
        let x = 240. - costume.rotation_center_x + x;
        let y = 180. - costume.rotation_center_y - y;
        costume.draw(x, y, self.direction, self.rotation_style);
        self.bubble.as_mut().map(|bubble| {
            let texture = costume.svg.get_texture();
            let boundary = Boundary {
                x: x - costume.rotation_center_x,
                y: y - costume.rotation_center_y,
                width: texture.width(),
                height: texture.height(),
            };
            bubble.draw(boundary, font);
        });
    }
    pub fn point_towards(&mut self, x: f32, y: f32) {
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
        bubble: None,
    };
    let arc = Arc::new(RwLock::new(sprite));
    Box::into_raw(Box::new(arc))
}

pub struct Scene {
    sprites: Vec<WrappedSprite>,
    pub cursor: RwLock<(f32, f32)>,
}

impl Scene {
    fn draw(&self, font: &Font) {
        for sprite in self.sprites.iter() {
            sprite.write().unwrap().draw(font);
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

async fn window_loop(scene: &Scene) {
    let font = load_ttf_font("helvetica.ttf").await.unwrap();
    loop {
        clear_background(color::WHITE);
        scene.draw(&font);
        next_frame().await
    }
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
