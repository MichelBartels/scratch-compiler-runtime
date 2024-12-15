use std::f32::consts::PI;
use std::ffi::{c_char, CStr};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::{Duration, Instant};

use resvg::usvg::Error::InvalidSize;

use macroquad::input::is_mouse_button_released;
use macroquad::math::Vec2;
use macroquad::text::{load_ttf_font, Font};
use macroquad::texture::{draw_texture_ex, DrawTextureParams};
use macroquad::{
    color, prelude::ImageFormat, texture::Texture2D, window::{clear_background, next_frame}, Window
};
use miniquad::MouseButton;

use crate::looks::{Boundary, Bubble};

const RES_SCALE: u32 = 4;

fn svg_to_texture(svg_str: &str) -> Option<Texture2D> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opt);
    let tree = match tree {
        Ok(tree) => tree,
        Err(InvalidSize) => {
            println!("Skipping SVG with invalid size");
            return None
        },
        Err(e) => panic!("Failed to parse SVG: {}", e),
    };
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width() * RES_SCALE, pixmap_size.height() * RES_SCALE).unwrap();

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(RES_SCALE as f32, RES_SCALE as f32),
        &mut pixmap.as_mut(),
    );
    let png = pixmap.encode_png().unwrap();
    Some(Texture2D::from_file_with_format(&png, Some(ImageFormat::Png)))
}

fn png_to_texture(png: &[u8]) -> Texture2D {
    Texture2D::from_file_with_format(png, Some(ImageFormat::Png))
}

#[derive(Clone)]
struct Texture {
    texture: Arc<Texture2D>,
    width: f32,
    height: f32
}

enum InnerLazyTexture {
    Loaded(Texture),
    SVG(String),
    PNG(Vec<u8>),
    None
}

struct LazyTexture {
    inner: Mutex<InnerLazyTexture>,
    cv: Condvar
}

impl LazyTexture {
    fn new(inner: InnerLazyTexture) -> Self {
        Self {
            inner: Mutex::new(inner),
            cv: Condvar::new(),
        }
    }
    fn load_texture(&self) {
        let texture = match &*self.inner.lock().unwrap() {
            InnerLazyTexture::Loaded(_) | InnerLazyTexture::None => panic!("Texture already loaded"),
            InnerLazyTexture::SVG(svg) => {
                svg_to_texture(&svg)
            },
            InnerLazyTexture::PNG(png) => {
                Some(png_to_texture(&png))
            }
        };
        match texture {
            Some (texture) => {
                let width = texture.width();
                let height = texture.height();
                *self.inner.lock().unwrap() = InnerLazyTexture::Loaded(Texture {
                    texture: Arc::new(texture),
                    width,
                    height
                });
            },
            None => {
                *self.inner.lock().unwrap() = InnerLazyTexture::None;
            }
        }
        self.cv.notify_all();
    }
    fn get_texture(&self) -> Option<Texture> {
        let texture = self.cv.wait_while(self.inner.lock().unwrap(), |inner| {
            !matches!(inner, InnerLazyTexture::Loaded(_)) && !matches!(inner, InnerLazyTexture::None)
        }).unwrap();
        match &*texture {
            InnerLazyTexture::Loaded(texture) => Some(texture.clone()),
            InnerLazyTexture::None => None,
            _ => panic!("Texture not loaded anymore"),
        }
    }
}

pub fn norm_angle(angle: f32) -> f32 {
    (angle + 180.0).rem_euclid(360.0) - 180.0
}

pub struct Costume {
    texture: LazyTexture,
    pub rotation_center_x: f32,
    pub rotation_center_y: f32,
    pub name: String,
}

impl Costume {
    fn rotation(&self, rotation: f32, rotation_style: RotationStyle) -> (f32, bool) {
        match rotation_style {
            RotationStyle::AllAround => ((rotation - 90.) * PI / 180.0, false),
            RotationStyle::LeftRight => (0.0, match norm_angle(rotation) {
                r if r < 0.0 => true,
                _ => false,
            }),
            RotationStyle::DontRotate => (0.0, false),
        }
    }
    fn draw(&mut self, x: f32, y: f32, rotation: f32, rotation_style: RotationStyle, scale: f32) {
        let (rotation, flip_x) = self.rotation(rotation, rotation_style);
        if let Some(texture) = self.texture.get_texture() {
            let texture = texture.texture;
            let size = texture.size() * scale / RES_SCALE as f32;
            draw_texture_ex(&texture, x, y, color::WHITE, DrawTextureParams {
                rotation,
                pivot: Some(Vec2 {
                    x: self.rotation_center_x * scale + x,
                    y: self.rotation_center_y * scale + y,

                }),
                flip_x,
                dest_size: Some(size),
                ..Default::default()
            })
        }
    }
}

#[no_mangle]
pub fn new_svg_costume(svg_str: *const c_char, x: f32, y: f32, name: *const c_char) -> *const Costume {
    let svg_str = unsafe { CStr::from_ptr(svg_str).to_str().unwrap().to_owned() };
    let name = unsafe { CStr::from_ptr(name).to_str().unwrap().to_owned() };
    let costume = Costume {
        texture: LazyTexture::new(InnerLazyTexture::SVG(svg_str.clone())),
        rotation_center_x: x,
        rotation_center_y: y,
        name,
    };
    Box::into_raw(Box::new(costume))
}

#[no_mangle]
pub fn new_png_costume(png: *const u8, len: i32, x: f32, y: f32, name: *const c_char) -> *const Costume {
    let png = unsafe { std::slice::from_raw_parts(png, len as usize) };
    let name = unsafe { CStr::from_ptr(name).to_str().unwrap().to_owned() };
    let costume = Costume {
        texture: LazyTexture::new(InnerLazyTexture::PNG(png.to_vec())),
        rotation_center_x: x,
        rotation_center_y: y,
        name,
    };
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
    pub scale: f32,
    pub shown: bool,
    pub index: usize,
    pub on_click: Vec<extern "C" fn()>
}

impl Sprite {
    fn load_textures(&self) {
        self.costumes.iter().for_each(|costume| {
            costume.texture.load_texture();
        });
    }
    fn boundary(&self) -> Boundary {
        let (x, y) = self.position.get_position();
        let costume = &self.costumes[self.current_costume];
        let x = 240. - costume.rotation_center_x * self.scale + x;
        let y = 180. - costume.rotation_center_y * self.scale - y;
        let (rotation, _) = costume.rotation(self.direction, self.rotation_style);
        match costume.texture.get_texture() {
            Some(texture) => Boundary {
                x,
                y,
                width: texture.width,
                height: texture.height,
                rotation,
            },
            None => Boundary {
                x,
                y,
                width: 0.0,
                height: 0.0,
                rotation,
            }
        }
    }
    fn draw(&mut self, font: &Font) {
        if !self.shown {
            return;
        }
        let boundary = self.boundary();
        let costume = &mut self.costumes[self.current_costume];
        costume.draw(boundary.x, boundary.y, self.direction, self.rotation_style, self.scale);
        self.bubble.as_mut().map(|bubble| {
            bubble.draw(boundary, font);
        });
    }
    pub fn point_towards(&mut self, x: f32, y: f32) {
        let (current_x, current_y) = self.position.get_position();
        let dx = x - current_x;
        let dy = y - current_y;
        self.direction = 90.0 - dy.atan2(dx).to_degrees();
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let boundary = self.boundary();
        boundary.contains(x + 240.0, 180.0 - y)
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
        scale: 1.0,
        shown: true,
        index: 0 as usize,
        on_click: Vec::new(),
    };
    let arc = Arc::new(RwLock::new(sprite));
    Box::into_raw(Box::new(arc))
}

#[no_mangle]
pub fn sprite_add_costume(sprite: *const WrappedSprite, costume: *mut Costume) -> i32 {
    let sprite = unsafe { &*sprite };
    let costume = unsafe { Box::from_raw(costume) };
    let mut sprite = sprite.write().unwrap();
    sprite.costumes.push(*costume);
    sprite.costumes.len() as i32 - 1

}

pub struct Scene {
    pub sprites: Vec<WrappedSprite>,
    pub cursor: (f32, f32),
}

impl Scene {
    fn draw(&mut self, font: &Font) {
        for sprite in self.sprites.iter() {
            sprite.write().unwrap().draw(font);
        }
        self.cursor = {
            let cursor = macroquad::input::mouse_position();
            (cursor.0 - 240.0, 180.0 - cursor.1)
        };
    }
}

pub type WrappedScene = RwLock<Scene>;

#[no_mangle]
pub fn new_scene() -> *const WrappedScene {
    Box::into_raw(Box::new(RwLock::new(Scene { sprites: Vec::new(), cursor: (0., 0.)})))
}

#[no_mangle]
pub fn scene_add_sprite(scene: *const WrappedScene, sprite: *const WrappedSprite) {
    let scene = unsafe { &*scene };
    let mut scene = scene.write().unwrap();
    let sprite = unsafe { &*sprite };
    sprite.write().unwrap().index = scene.sprites.len();
    scene.sprites.push(sprite.clone());
}

async fn window_loop(scene: &WrappedScene) {
    let font = load_ttf_font("helvetica.ttf").await.unwrap();
    scene.read().unwrap().sprites.iter().for_each(|sprite| {
        let sprite = sprite.read().unwrap();
        sprite.load_textures();
    });
    loop {
        clear_background(color::WHITE);
        {
            scene.write().unwrap().draw(&font);
        }
        let cursor = {
            scene.read().unwrap().cursor
        };
        if is_mouse_button_released(MouseButton::Left) {
            scene.read().unwrap().sprites.iter().filter(|sprite| {
                let sprite = sprite.read().unwrap();
                sprite.contains(cursor.0, cursor.1) && sprite.shown
            }).max_by_key(|sprite| {
                let sprite = sprite.read().unwrap();
                sprite.index
            }).map(|sprite| {
                let sprite = sprite.read().unwrap();
                sprite.on_click.iter().for_each(|f| {
                    let f = *f;
                    std::thread::spawn(move || f());
                });
            });
        }
        next_frame().await
    }
}

#[no_mangle]
pub fn create_window(scene: *const WrappedScene) {
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
