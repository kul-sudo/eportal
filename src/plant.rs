use crate::{
    constants::*, Body, BodyId, Cell, Zoom, AREA_SIZE, CELLS,
};
use macroquad::{
    color::{GREEN, YELLOW},
    math::Vec2,
    prelude::vec2,
    shapes::{draw_triangle, draw_triangle_lines},
};
use rand::{prelude::IteratorRandom, rngs::StdRng, Rng};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq)]
pub enum PlantKind {
    Grass,
    Banana,
}

impl PlantKind {
    pub const ALL: [Self; 2] = [Self::Grass, Self::Banana];
}

#[derive(PartialEq, Clone)]
pub struct Plant {
    pub pos:         Vec2,
    pub kind:        PlantKind,
    pub followed_by: HashMap<BodyId, Body>,
}

pub type PlantId = Instant;

impl Plant {
    #[inline(always)]
    pub fn draw(&self) {
        match self.kind {
            PlantKind::Grass => {
                draw_triangle_lines(
                    vec2(self.pos.x, self.pos.y - OBJECT_RADIUS),
                    vec2(
                        self.pos.x
                            + OBJECT_RADIUS * COSINE_OF_30_DEGREES,
                        self.pos.y + OBJECT_RADIUS / 2.0,
                    ),
                    vec2(
                        self.pos.x
                            - OBJECT_RADIUS * COSINE_OF_30_DEGREES,
                        self.pos.y + OBJECT_RADIUS / 2.0,
                    ),
                    2.0,
                    GREEN,
                );
            }
            PlantKind::Banana => {
                draw_triangle(
                    vec2(self.pos.x, self.pos.y - OBJECT_RADIUS),
                    vec2(
                        self.pos.x
                            + OBJECT_RADIUS * COSINE_OF_30_DEGREES,
                        self.pos.y + OBJECT_RADIUS / 2.0,
                    ),
                    vec2(
                        self.pos.x
                            - OBJECT_RADIUS * COSINE_OF_30_DEGREES,
                        self.pos.y + OBJECT_RADIUS / 2.0,
                    ),
                    YELLOW,
                );
            }
        }
    }

    #[inline(always)]
    pub fn get_contained_energy(&self) -> f32 {
        match self.kind {
            PlantKind::Grass => GRASS_ENERGY,
            PlantKind::Banana => BANANA_ENERGY,
        }
    }

    #[inline(always)]
    /// Get the plants needed to be drawn.
    pub fn get_plants_to_draw<'a>(
        zoom: &'a Zoom,
        plants: &'a [Vec<HashMap<PlantId, Plant>>],
        removed_plants: &'a HashMap<PlantId, Vec2>,
        plants_n: usize,
    ) -> Vec<&'a Self> {
        let mut plants_to_draw = Vec::with_capacity(
            (plants_n as f32 * AVERAGE_PLANTS_PART_DRAWN) as usize,
        );

        let (i_min, i_max, j_min, j_max);

        if let Some(extended_rect) = zoom.extended_rect {
            let extended_rect_center = extended_rect.center();

            i_min = ((extended_rect_center.y - extended_rect.h / 2.0)
                / CELLS.cell_height)
                .floor() as usize;
            i_max = ((extended_rect_center.y + extended_rect.h / 2.0)
                / CELLS.cell_height)
                .floor() as usize;
            j_min = ((extended_rect_center.x - extended_rect.w / 2.0)
                / CELLS.cell_width)
                .floor() as usize;
            j_max = ((extended_rect_center.x + extended_rect.w / 2.0)
                / CELLS.cell_width)
                .floor() as usize;
        } else {
            unreachable!()
        }

        for (i, row) in plants
            .iter()
            .enumerate()
            .take(i_max.min(CELLS.rows - 1) + 1)
            .skip(i_min.max(0))
        {
            let i_fully_within_rectangle = i != i_min && i != i_max;

            for (j, plants) in row
                .iter()
                .enumerate()
                .take(j_max.min(CELLS.columns - 1) + 1)
                .skip(j_min.max(0))
            {
                let j_fully_within_rectangle =
                    j != j_min && j != j_max;
                if i_fully_within_rectangle
                    && j_fully_within_rectangle
                {
                    // The cell is fully within the rectangle
                    for (plant_id, plant) in plants {
                        if !removed_plants.contains_key(plant_id) {
                            plants_to_draw.push(plant);
                        }
                    }
                } else {
                    for (plant_id, plant) in plants {
                        if !removed_plants.contains_key(plant_id)
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
        bodies: &HashMap<BodyId, Body>,
        plants: &mut [Vec<HashMap<PlantId, Self>>],
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
            pos.x = rng.gen_range(0.0..AREA_SIZE.x);
            pos.y = rng.gen_range(0.0..AREA_SIZE.y);
            (pos.x <= OBJECT_RADIUS + MIN_GAP
                || pos.x >= AREA_SIZE.x - OBJECT_RADIUS - MIN_GAP)
                || (pos.y <= OBJECT_RADIUS + MIN_GAP
                    || pos.y >= AREA_SIZE.y - OBJECT_RADIUS - MIN_GAP)
                || bodies.values().any(|body| {
                    body.pos.distance(pos)
                        <= OBJECT_RADIUS * 2.0 + MIN_GAP
                })
        } {}

        let Cell { i, j } = CELLS.get_cell_by_pos(&pos);
        plants[i][j].insert(
            Instant::now(),
            Self {
                pos,
                kind: *PlantKind::ALL.iter().choose(rng).unwrap(),
                followed_by: HashMap::new(),
            },
        );
    }
}
