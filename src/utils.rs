use crate::constants::*;
use crate::user_constants::*;
use crate::Condition;
use crate::Zoom;
use ::rand::{rngs::StdRng, Rng};
use macroquad::prelude::*;
use std::time::{Duration, Instant};

pub struct LastInfo {
    pub plants_n: usize,
    pub bodies_n: usize,
}

pub struct EvolutionInfo {
    pub show:         bool,
    pub last_updated: Option<Instant>,
    pub last_info:    Option<LastInfo>,
}

pub struct Info {
    pub body_info:      bool,
    pub evolution_info: EvolutionInfo,
}

pub fn generate_zoom_struct(
    area_size: &Vec2,
    rect_size: &Vec2,
) -> Zoom {
    let scaling_width = MAX_ZOOM / area_size.x * 2.0;
    let scaling_height = MAX_ZOOM / area_size.y * 2.0;

    Zoom {
        zoomed: false,
        scaling_width,
        scaling_height,
        center_pos: None,
        mouse_pos: None,
        rect: None,
        extended_rect: None,
    }
}

#[inline(always)]
pub fn show_evolution_info(
    zoom: &Zoom,
    area_size: &Vec2,
    info: &mut Info,
    plants_info: (usize, usize),
    bodies_info: (usize, usize),
    condition: &Option<(Condition, (Instant, Duration))>,
) {
    let (plants_n, removed_plants_len) = plants_info;
    let (bodies_n, removed_bodies_len) = bodies_info;

    let real_plants_n = plants_n - removed_plants_len;
    let real_bodies_n = bodies_n - removed_bodies_len;

    let plants_n_to_show;
    let bodies_n_to_show;

    let update_evolution_info = info
        .evolution_info
        .last_updated
        .unwrap()
        .elapsed()
        .as_secs_f32()
        > 0.5;

    match info.evolution_info.last_info {
        Some(_) => {
            if update_evolution_info {
                let LastInfo {
                    plants_n: last_plants_n,
                    bodies_n: last_bodies_n,
                } = info.evolution_info.last_info.as_mut().unwrap();

                *last_plants_n = real_plants_n;
                *last_bodies_n = real_bodies_n;

                plants_n_to_show = real_plants_n;
                bodies_n_to_show = real_bodies_n;

                info.evolution_info.last_updated =
                    Some(Instant::now());
            } else {
                let LastInfo {
                    plants_n: last_plants_n,
                    bodies_n: last_bodies_n,
                } = info.evolution_info.last_info.as_ref().unwrap();

                plants_n_to_show = *last_plants_n;
                bodies_n_to_show = *last_bodies_n;
            }
        }
        None => {
            info.evolution_info.last_info = Some({
                LastInfo {
                    plants_n: real_plants_n,
                    bodies_n: real_bodies_n,
                }
            });

            plants_n_to_show = real_plants_n;
            bodies_n_to_show = real_bodies_n;

            info.evolution_info.last_updated = Some(Instant::now());
        }
    }

    let evolution_info_fields = [
        format!("plants: {:?}", plants_n_to_show),
        format!("bodies: {:?}", bodies_n_to_show),
        format!(
            "condition: {}",
            match condition {
                Some((condition, _)) => {
                    format!("{:?}", condition)
                }
                None => {
                    "Normal".to_string()
                }
            }
        ),
    ];

    let mut gap = 0.0;

    if zoom.zoomed {
        for field in evolution_info_fields {
            let evolution_info_font_size =
                (EVOLUTION_INFO_FONT_SIZE as f32 / MAX_ZOOM) as u16;
            let measured = measure_text(
                &field,
                None,
                evolution_info_font_size,
                1.0,
            );

            draw_text(
                &field,
                zoom.rect.unwrap().x + zoom.rect.unwrap().w
                    - measured.width,
                zoom.rect.unwrap().y + measured.offset_y + gap,
                evolution_info_font_size as f32,
                WHITE,
            );

            gap += measured.offset_y + EVOLUTION_INFO_GAP / MAX_ZOOM;
        }
    } else {
        for field in evolution_info_fields {
            let measured = measure_text(
                &field,
                None,
                EVOLUTION_INFO_FONT_SIZE,
                1.0,
            );

            draw_text(
                &field,
                area_size.x - measured.width,
                measured.offset_y + gap,
                EVOLUTION_INFO_FONT_SIZE as f32,
                WHITE,
            );

            gap += measured.offset_y + EVOLUTION_INFO_GAP;
        }
    }
}

#[inline(always)]
pub fn show_fps(zoom: &Zoom) {
    let text = format!(
        "{:?}",
        ((get_fps() as f32 / 5.0).round() * 5.0) as usize
    );

    if zoom.zoomed {
        let font_size = (FPS_FONT_SIZE as f32 / MAX_ZOOM) as u16;

        let measured = measure_text(&text, None, font_size, 1.0);

        draw_text(
            &text,
            zoom.rect.unwrap().x,
            zoom.rect.unwrap().y + measured.height,
            font_size as f32,
            WHITE,
        );
    } else {
        let measured = measure_text(&text, None, FPS_FONT_SIZE, 1.0);

        draw_text(
            &text,
            0.0,
            measured.height,
            FPS_FONT_SIZE as f32,
            WHITE,
        );
    }
}

/// Adjust the coordinates according to the borders.
#[inline(always)]
pub fn adjusted_pos(pos: &Vec2, area_size: &Vec2) -> Vec2 {
    vec2(
        (pos.x * MAX_ZOOM)
            .max(area_size.x / MAX_ZOOM / 2.0)
            .min(area_size.x * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        (pos.y * MAX_ZOOM)
            .max(area_size.y / MAX_ZOOM / 2.0)
            .min(area_size.y * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
    )
}

/// Used for getting specific values with deviations.
#[inline(always)]
pub fn get_with_deviation(value: f32, rng: &mut StdRng) -> f32 {
    let part = value * unsafe { DEVIATION };
    rng.gen_range(value - part..value + part)
}
