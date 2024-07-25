use crate::body::Skill;
use crate::constants::*;
use crate::user_constants::*;
use crate::Condition;
use crate::UI_SHOW_PROPERTIES_N;
use crate::{Virus, Zoom};
use crate::{TOTAL_SKILLS_COUNT, VIRUSES_COUNT};
use ::rand::{rngs::StdRng, Rng};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem::variant_count;
use std::time::{Duration, Instant};

pub fn enum_consts() -> (HashSet<usize>, HashSet<usize>) {
    unsafe {
        UI_SHOW_PROPERTIES_N = (size_of::<UIField>()
            - size_of::<u16>())
            / size_of::<bool>();
    }

    // Skills
    let mut variant_count_ = variant_count::<Skill>();
    unsafe {
        TOTAL_SKILLS_COUNT = variant_count_;
    }
    let all_skills = (0..variant_count_).collect::<HashSet<_>>();

    // Viruses
    variant_count_ = variant_count::<Virus>();
    unsafe {
        VIRUSES_COUNT = variant_count_;
    }
    let all_viruses = (0..variant_count_).collect::<HashSet<_>>();

    (all_skills, all_viruses)
}

pub fn generate_zoom_struct(
    area_size: &Vec2,
    rect_size: &Vec2,
) -> Zoom {
    let scaling_width = MAX_ZOOM / area_size.x * 2.0;
    let scaling_height = MAX_ZOOM / area_size.y * 2.0;

    let extended_rect_width = rect_size.x + OBJECT_RADIUS * 2.0;
    let extended_rect_height = rect_size.y + OBJECT_RADIUS * 2.0;

    Zoom {
        zoomed: false,
        scaling_width,
        scaling_height,
        center_pos: None,
        mouse_pos: None,
        rect: None,
        extended_rect: None,
        diagonal_rect: (rect_size.x.powi(2) + rect_size.y.powi(2))
            .sqrt(),
        diagonal_extended_rect: (extended_rect_width.powi(2)
            + extended_rect_height.powi(2))
        .sqrt(),
    }
}

#[inline(always)]
pub fn show_evolution_info(
    zoom: &Zoom,
    area_size: &Vec2,
    plants_n: usize,
    removed_plants_len: usize,
    bodies_len: usize,
    removed_bodies_len: usize,
    conditions: &HashMap<Condition, (Instant, Duration)>,
) {
    let evolution_info_fields = [
        format!("plants: {:?}", plants_n - removed_plants_len),
        format!("bodies: {:?}", bodies_len - removed_bodies_len),
        format!("conditions: {:?}", conditions.keys()),
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
