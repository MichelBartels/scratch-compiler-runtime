use macroquad::{
    color::{colors, Color},
    shapes::{draw_arc, draw_circle_lines, draw_line},
    text::draw_text,
    text::{measure_text, Font},
};
use unicode_linebreak::{linebreaks, BreakOpportunity};

use crate::ui::WrappedSprite;

fn measure_width(text: &str, font: &Font) -> f32 {
    measure_text(text, Some(font), 16, 1.0).width
}

fn wrap_simple<'a>(text: &'a str, max_width: f32, font: &Font) -> Vec<&'a str> {
    let mut lines = Vec::new();
    let mut current_start = 0;

    while measure_width(&text[current_start..], font) > max_width {
        let mut split_pos = 0;
        for (i, _) in text[current_start..].char_indices() {
            if measure_width(&text[current_start..split_pos], font) > max_width {
                break;
            }
            split_pos = i;
        }
        lines.push(&text[current_start..current_start + split_pos]);
        current_start += split_pos;
    }
    if current_start < text.len() {
        lines.push(&text[current_start..]);
    }
    lines
}

fn wrap_text<'a>(text: &'a str, max_width: f32, font: &Font) -> Vec<&'a str> {
    let linebreaks = linebreaks(text);
    let mut lines = Vec::new();
    let mut current_start = 0;
    let mut previous_end = None;

    for (pos, break_opportunity) in linebreaks {
        let mut current_line = &text[current_start..pos];
        if measure_width(&current_line, font) > max_width {
            if let Some(previous_end) = previous_end {
                let previous_line = &text[current_start..previous_end];
                lines.push(previous_line);
                current_start = previous_end;
                current_line = &text[current_start..pos];
            }
        }
        if measure_width(&current_line, font) > max_width
            || break_opportunity == BreakOpportunity::Mandatory
        {
            // a bit ugly
            lines.extend(wrap_simple(&current_line, max_width, font));
            current_start = pos;
            previous_end = None;
        } else {
            previous_end = Some(pos);
        }
    }
    lines
}

enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub enum BubbleType {
    Speech,
    Thought,
}

pub struct Bubble {
    pub text: String,
    pub bubble_type: BubbleType,
    pub lines: Option<Vec<String>>,
}

fn max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

fn min(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub struct Boundary {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bubble {
    pub fn draw(&mut self, boundary: Boundary, font: &Font) {
        let line_height = 16.0;
        let wrapped_text = self.lines.get_or_insert_with(|| {
            wrap_text(&self.text, 170.0, font)
                .iter()
                .map(|s| s.to_string())
                .collect()
        });
        let max_width = wrapped_text
            .iter()
            .map(|line| measure_width(line, font))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let padded_width = max_width + 20.0;
        let padded_height = line_height * wrapped_text.len() as f32 + 20.0;

        let direction = if padded_width + boundary.x + boundary.width > 480.0 {
            Direction::Left
        } else {
            Direction::Right
        };

        let x = min(
            240.0 - padded_width,
            max(
                -240.0,
                match direction {
                    Direction::Left => boundary.x - padded_width,
                    Direction::Right => boundary.x + boundary.width,
                },
            ),
        );
        let y = min(180.0, boundary.y);

        for (i, line) in wrapped_text.iter().enumerate() {
            draw_text(
                line,
                x + 10.0,
                y + 10.0 + 10.0 + i as f32 * line_height,
                14.0,
                colors::BLACK,
            );
        }
        let stroke_width = 2.0;
        let corner_radius = 16.0;
        let corner_radius_inner = corner_radius - stroke_width / 2.0;
        let arc_res = 200;
        let border_colour = Color::new(0.0, 0.0, 0.0, 0.15);
        draw_arc(
            x + corner_radius,
            y + padded_height - corner_radius,
            arc_res,
            corner_radius_inner,
            90.0,
            stroke_width,
            90.0,
            border_colour,
        );
        draw_line(
            x,
            y + padded_height - corner_radius,
            x,
            y + corner_radius,
            stroke_width,
            border_colour,
        );
        draw_arc(
            x + corner_radius,
            y + corner_radius,
            arc_res,
            corner_radius_inner,
            180.0,
            stroke_width,
            90.0,
            border_colour,
        );
        draw_line(
            x + corner_radius,
            y,
            x + padded_width - corner_radius,
            y,
            stroke_width,
            border_colour,
        );
        draw_arc(
            x + padded_width - corner_radius,
            y + corner_radius,
            arc_res,
            corner_radius_inner,
            270.0,
            stroke_width,
            90.0,
            border_colour,
        );
        draw_line(
            x + padded_width,
            y + corner_radius,
            x + padded_width,
            y + padded_height - corner_radius,
            stroke_width,
            border_colour,
        );
        draw_arc(
            x + padded_width - corner_radius,
            y + padded_height - corner_radius,
            arc_res,
            corner_radius_inner,
            0.0,
            stroke_width,
            90.0,
            border_colour,
        );

        match (direction, self.bubble_type) {
            (Direction::Left, BubbleType::Speech) => {
                draw_line(
                    x + padded_width - corner_radius,
                    y + padded_height,
                    x + padded_width - corner_radius + 3.0,
                    y + padded_height + 14.0,
                    stroke_width,
                    border_colour,
                );
                draw_line(
                    x + padded_width - corner_radius + 3.0,
                    y + padded_height + 14.0,
                    x + padded_width - corner_radius - 16.0,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
                draw_line(
                    x + padded_width - corner_radius - 16.0,
                    y + padded_height,
                    x + corner_radius,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
            }
            (Direction::Left, BubbleType::Thought) => {
                draw_line(
                    x + padded_width - corner_radius,
                    y + padded_height,
                    x + padded_width - corner_radius - 12.0,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
                draw_arc(
                    x + padded_width - corner_radius - 16.0 - stroke_width / 2.0,
                    y + padded_height - stroke_width / 2.0,
                    arc_res,
                    4.0 - stroke_width / 2.0,
                    0.0,
                    stroke_width,
                    180.0,
                    border_colour,
                );
                draw_circle_lines(
                    x + padded_width - corner_radius - 9.25,
                    y + padded_height + 7.25,
                    2.25,
                    stroke_width,
                    border_colour,
                );
                draw_circle_lines(
                    x + padded_width - corner_radius - 1.5,
                    y + padded_height + 9.5,
                    1.5,
                    stroke_width,
                    border_colour,
                );
                draw_line(
                    x + padded_width - corner_radius - 20.0,
                    y + padded_height,
                    x + corner_radius,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
            }
            (Direction::Right, BubbleType::Speech) => {
                draw_line(
                    x + corner_radius,
                    y + padded_height,
                    x + corner_radius - 3.0,
                    y + padded_height + 14.0,
                    stroke_width,
                    border_colour,
                );
                draw_line(
                    x + corner_radius - 3.0,
                    y + padded_height + 14.0,
                    x + corner_radius + 16.0,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
                draw_line(
                    x + corner_radius + 16.0,
                    y + padded_height,
                    x + padded_width - corner_radius,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
            }
            (Direction::Right, BubbleType::Thought) => {
                draw_line(
                    x + corner_radius,
                    y + padded_height,
                    x + corner_radius + 12.0,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
                draw_arc(
                    x + corner_radius + 16.0 + stroke_width / 2.0,
                    y + padded_height - stroke_width / 2.0,
                    arc_res,
                    4.0 - stroke_width / 2.0,
                    0.0,
                    stroke_width,
                    180.0,
                    border_colour,
                );
                draw_circle_lines(
                    x + corner_radius + 9.25,
                    y + padded_height + 7.25,
                    2.25,
                    stroke_width,
                    border_colour,
                );
                draw_circle_lines(
                    x + corner_radius + 1.5,
                    y + padded_height + 9.5,
                    1.5,
                    stroke_width,
                    border_colour,
                );
                draw_line(
                    x + corner_radius + 20.0,
                    y + padded_height,
                    x + padded_width - corner_radius,
                    y + padded_height,
                    stroke_width,
                    border_colour,
                );
            }
        };
    }
}

#[no_mangle]
pub fn looks_say(sprite: *const WrappedSprite, text: *const String) {
    let sprite = unsafe { &*sprite };
    let text = unsafe { &*text };
    let bubble = Bubble {
        text: text.clone(),
        bubble_type: BubbleType::Speech,
        lines: None,
    };
    let mut sprite = sprite.write().unwrap();
    sprite.bubble = Some(bubble);
}
