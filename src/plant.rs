use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use macroquad::{
    color::GREEN, math::Vec2, prelude::vec2, shapes::draw_triangle,
};
use rand::{rngs::StdRng, Rng};

use crate::{constants::*, zoom::Zoom, Body, Cell, Cells};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plant {
    pub pos: Vec2,
}

impl Plant {
    #[inline(always)]
    pub fn draw(&self) {
        draw_triangle(
            vec2(self.pos.x, self.pos.y - OBJECT_RADIUS),
            vec2(
                self.pos.x + OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                self.pos.y + OBJECT_RADIUS / 2.0,
            ),
            vec2(
                self.pos.x - OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                self.pos.y + OBJECT_RADIUS / 2.0,
            ),
            GREEN,
        );
    }

    #[inline(always)]
    /// Get the plants needed to be drawn.
    pub fn get_plants_to_draw<'a>(
        cells: &'a Cells,
        zoom: &'a Zoom,
        plants: &'a HashMap<Cell, HashMap<Instant, Plant>>,
        removed_plants: &'a [(Instant, Vec2)],
    ) -> Vec<&'a Plant> {
        let mut plants_to_draw = Vec::new();

        let extended_rect_center =
            zoom.extended_rect.unwrap().center();

        let i_min = ((extended_rect_center.y
            - extended_rect_center.y)
            / cells.cell_height)
            .floor() as usize;
        let i_max = ((extended_rect_center.y
            + extended_rect_center.y)
            / cells.cell_height)
            .floor() as usize;
        let j_min = ((extended_rect_center.x
            - extended_rect_center.x)
            / cells.cell_width)
            .floor() as usize;
        let j_max = ((extended_rect_center.x
            + extended_rect_center.x)
            / cells.cell_width)
            .floor() as usize;

        for i in i_min.max(0)..=i_max.min(cells.rows - 1) {
            let i_is_on_border = i == i_min || i == i_max;

            for j in j_min.max(0)..=j_max.min(cells.columns - 1) {
                if !i_is_on_border && (j != j_min && j != j_max) {
                    // The cell is fully within the rectangle
                    for (plant_id, plant) in
                        plants.get(&Cell { i, j }).unwrap()
                    {
                        if !removed_plants
                            .contains(&(*plant_id, plant.pos))
                        {
                            plants_to_draw.push(plant);
                        }
                    }
                } else {
                    for (plant_id, plant) in
                        plants.get(&Cell { i, j }).unwrap()
                    {
                        if !removed_plants
                            .contains(&(*plant_id, plant.pos))
                            && zoom
                                .extended_rect
                                .unwrap()
                                .contains(plant.pos)
                        {
                            plants_to_draw.push(plant);
                        }
                    }
                }
            }
        }

        plants_to_draw
    }

    #[inline(always)]
    /// Spawn a plant to a random position on the field.
    pub fn randomly_spawn_plant(
        bodies: &HashMap<Instant, Body>,
        plants: &mut HashMap<Cell, HashMap<Instant, Plant>>,
        area_size: &Vec2,
        cells: &Cells,
        rng: &mut StdRng,
    ) {
        let mut pos = Vec2::default();

        let starting_point = Instant::now();

        // Make sure the position is far enough from the rest of the plants and bodies and the borders of the area
        while {
            // Make sure finding a suitable position doesn't exceed a specific time limit
            if starting_point.elapsed().as_nanos()
                >= Duration::from_millis(PLANT_SPAWN_TIME_LIMIT)
                    .as_nanos()
            {
                return;
            }
            pos.x = rng.gen_range(0.0..area_size.x);
            pos.y = rng.gen_range(0.0..area_size.y);
            (pos.x <= OBJECT_RADIUS + MIN_GAP
                || pos.x >= area_size.x - OBJECT_RADIUS - MIN_GAP)
                || (pos.y <= OBJECT_RADIUS + MIN_GAP
                    || pos.y >= area_size.y - OBJECT_RADIUS - MIN_GAP)
                || plants
                    .get_mut(&cells.get_cell_by_pos(&pos))
                    .unwrap()
                    .len()
                    >= AVERAGE_MAX_PLANTS_IN_ONE_CELL
                || bodies.values().any(|body| {
                    body.pos.distance(pos)
                        <= OBJECT_RADIUS * 2.0 + MIN_GAP
                })
        } {}

        plants
            .get_mut(&cells.get_cell_by_pos(&pos))
            .unwrap()
            .insert(Instant::now(), Plant { pos });
    }
}
