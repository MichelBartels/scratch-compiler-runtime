use std::ffi::{c_char, CStr};
use std::sync::RwLock;

use macroquad::{
    color, prelude::ImageFormat, texture::{draw_texture, Texture2D}, window::{clear_background, next_frame}, Window
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

#[derive(Clone, Debug)]
pub struct Sprite {
    texture: Option<Texture2D>,
    svg: String,
    x: i32,
    y: i32,
}

impl Sprite {
    fn new(texture: Option<Texture2D>, svg: String, x: i32, y: i32) -> Self {
        Self { texture, svg, x, y }
    }

    fn get_texture(&mut self) -> Texture2D {
        if self.texture.is_none() {
            self.texture = Some(svg_to_texture(&self.svg));
        }
        self.texture.clone().unwrap()
    }

    fn draw(&mut self) {
        draw_texture(&self.get_texture(), self.x as f32, self.y as f32, color::WHITE);
    }
}

#[no_mangle]
pub fn new_sprite(svg_str: *const c_char, x: i32, y: i32) -> *mut Sprite {
    let svg = unsafe { CStr::from_ptr(svg_str).to_str().unwrap() };
    let sprite = Sprite::new(None, svg.to_owned(), x, y);
    Box::into_raw(Box::new(sprite))
}

pub struct Scene {
    sprites: RwLock<Vec<Sprite>>,
}

impl Scene {
    fn new() -> Self {
        Self { sprites: RwLock::new(Vec::new()) }
    }

    fn add_sprite(&self, sprite: Sprite) {
        self.sprites.write().unwrap().push(sprite);
    }

    fn draw(&self) {
        for sprite in self.sprites.write().unwrap().iter_mut() {
            sprite.draw();
        }
    }
}

#[no_mangle]
pub fn new_scene() -> *const Scene {
    println!("Creating new scene");
    let ptr = Box::into_raw(Box::new(Scene::new()));
    println!("Scene: {:?}", ptr);
    ptr
}

#[no_mangle]
pub fn scene_add_sprite(scene: *const Scene, sprite: *const Sprite) {
    println!("Adding sprite to scene");
    let scene = unsafe { &*scene };
    let sprite = unsafe { &*sprite };
    scene.add_sprite(sprite.clone());
}

#[no_mangle]
pub fn create_window(scene: *const Scene) {
    println!("Creating window");
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
