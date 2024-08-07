use crate::{
    constants::*,
    get_with_deviation,
    smart_drawing::{DrawingStrategy, RectangleCorner},
    user_constants::*,
    Body, BodyId, Cell, Cells, Plant, PlantId, PlantKind, Virus,
    Zoom, UI_SHOW_PROPERTIES_N,
};
use macroquad::prelude::{
    draw_circle, draw_line, draw_rectangle, draw_text, measure_text,
    rand::gen_range, vec2, Circle, Color, Vec2, Vec3, GREEN, RED,
    WHITE,
};
use rand::{random, rngs::StdRng, seq::IteratorRandom, Rng};
use std::{
    collections::HashMap, collections::HashSet, f32::consts::PI,
    f32::consts::SQRT_2, time::Instant,
};

pub type CrossId = Instant;

#[derive(Clone, PartialEq)]
pub struct Cross {
    pub pos:         Vec2,
    pub timestamp:   Instant,
    pub energy:      f32,
    pub viruses:     HashMap<Virus, f32>,
    pub color:       Color,
    pub body_type:   u16,
    pub followed_by: HashMap<BodyId, Body>,
}

impl Cross {
    pub fn new(body: &Body) -> Self {
        Self {
            pos:         body.pos,
            timestamp:   Instant::now(),
            energy:      body.energy,
            viruses:     body.viruses.clone(),
            color:       body.color,
            body_type:   body.body_type,
            followed_by: body.followed_by.clone(),
        }
    }

    pub fn draw(&self, zoom: &Zoom) {
        if zoom.zoomed
            && !zoom.extended_rect.unwrap().contains(self.pos)
        {
            return;
        }

        let side_length_half = OBJECT_RADIUS / SQRT_2;

        draw_line(
            self.pos.x - side_length_half,
            self.pos.y - side_length_half,
            self.pos.x + side_length_half,
            self.pos.y + side_length_half,
            2.0,
            self.color,
        );

        draw_line(
            self.pos.x + side_length_half,
            self.pos.y - side_length_half,
            self.pos.x - side_length_half,
            self.pos.y + side_length_half,
            2.0,
            self.color,
        );

        if !self.viruses.is_empty() {
            draw_circle(self.pos.x, self.pos.y, 5.0, RED)
        }
    }
}
