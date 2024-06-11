use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use macroquad::{color::GREEN, math::Vec2, shapes::draw_triangle};
use rand::{rngs::StdRng, Rng};

use crate::{
    zoom::Zoom, Body, Cell, Cells, COSINE_OF_30_DEGREES, MIN_GAP, OBJECT_RADIUS,
    PLANT_SPAWN_TIME_LIMIT,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plant {
    pub pos: Vec2,
}

impl Plant {
    pub fn draw(&self) {
        draw_triangle(
            Vec2 {
                x: self.pos.x,
                y: self.pos.y - OBJECT_RADIUS,
            },
            Vec2 {
                x: self.pos.x + OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: self.pos.y + OBJECT_RADIUS / 2.0,
            },
            Vec2 {
                x: self.pos.x - OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: self.pos.y + OBJECT_RADIUS / 2.0,
            },
            GREEN,
        );
    }

    pub fn get_plants_to_draw(
        cells: &Cells,
        zoom: &Zoom,
        plants: &HashMap<Cell, HashMap<Instant, Plant>>,
        removed_plants: &[(Instant, Vec2)],
    ) -> Vec<Plant> {
        let mut plants_to_draw = Vec::new();
        let (i_min, i_max, j_min, j_max);

        if let Some(extended_rect) = zoom.extended_rect {
            i_min = ((extended_rect.center().y - extended_rect.h / 2.0) / cells.cell_height).floor()
                as usize;
            i_max = ((extended_rect.center().y + extended_rect.h / 2.0) / cells.cell_height).floor()
                as usize;
            j_min = ((extended_rect.center().x - extended_rect.w / 2.0) / cells.cell_width).floor()
                as usize;
            j_max = ((extended_rect.center().x + extended_rect.w / 2.0) / cells.cell_width).floor()
                as usize;
        } else {
            unreachable!()
        }

        for i in i_min.max(0)..=i_max.min(cells.rows - 1) {
            let i_is_on_border = i == i_min || i == i_max;

            for j in j_min.max(0)..=j_max.min(cells.columns - 1) {
                if !i_is_on_border && (j != j_min && j != j_max) {
                    // The cell is fully within the rectangle
                    for (plant_id, plant) in plants.get(&Cell { i, j }).unwrap() {
                        if !removed_plants.contains(&(*plant_id, plant.pos)) {
                            plants_to_draw.push(*plant);
                        }
                    }
                } else {
                    for (plant_id, plant) in plants.get(&Cell { i, j }).unwrap() {
                        if !removed_plants.contains(&(*plant_id, plant.pos))
                            && zoom.extended_rect.unwrap().contains(plant.pos)
                        {
                            plants_to_draw.push(*plant);
                        }
                    }
                }
            }
        }

        plants_to_draw
    }
}

pub fn randomly_spawn_plant(
    bodies: &HashMap<Instant, Body>,
    plants: &mut HashMap<Cell, HashMap<Instant, Plant>>,
    area_size: &Vec2,
    cells: &Cells,
    rng: &mut StdRng,
) {
    let starting_point = Instant::now();

    let mut pos = Vec2::default();

    // Make sure the position is far enough from the rest of the plants and bodies and the borders of the area
    while {
        // Make sure finding a suitable position doesn't exceed a specific time limit
        if starting_point.elapsed().as_nanos()
            >= Duration::from_millis(PLANT_SPAWN_TIME_LIMIT).as_nanos()
        {
            return;
        }
        pos.x = rng.gen_range(0.0..area_size.x);
        pos.y = rng.gen_range(0.0..area_size.y);
        (pos.x <= OBJECT_RADIUS + MIN_GAP || pos.x >= area_size.x - OBJECT_RADIUS - MIN_GAP)
            || (pos.y <= OBJECT_RADIUS + MIN_GAP || pos.y >= area_size.y - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| body.pos.distance(pos) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    plants
        .get_mut(&cells.get_cell_by_pos(&pos))
        .unwrap()
        .insert(Instant::now(), Plant { pos });
}
