use crate::{
    constants::*,
    get_with_deviation,
    smart_drawing::{DrawingStrategy, RectangleCorner},
    user_constants::*,
    Cell, Cross, CrossId, Plant, PlantId, PlantKind, Zoom, AREA_SIZE,
    CELLS,
};
use macroquad::prelude::{
    draw_circle, draw_rectangle, draw_rectangle_ex, draw_text,
    measure_text, rand::gen_range, vec2, Circle, Color,
    DrawRectangleParams, Vec2, Vec3, GREEN, RED, WHITE,
};
use rand::{random, rngs::StdRng, seq::IteratorRandom, Rng};
use std::{
    collections::HashMap, collections::HashSet, f32::consts::PI,
    f32::consts::SQRT_2, time::Instant,
};

#[derive(Copy, Clone, PartialEq)]
pub enum ObjectType {
    Body,
    Plant,
    Cross,
}

#[derive(Copy, Clone)]
pub struct FoodInfo<'a> {
    pub id:        Instant,
    pub food_type: ObjectType,
    pub pos:       Vec2,
    pub energy:    f32,
    pub viruses:   Option<&'a HashMap<Virus, f32>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    FollowingTarget(Instant, Vec2, ObjectType),
    EscapingBody(BodyId, u32),
    Walking(Vec2),
    Cross,
    Undefined,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EatingStrategy {
    Omnivorous,
    Herbivorous,
    Carnivorous,
}

#[allow(dead_code)]
#[repr(usize)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
/// https://github.com/kul-sudo/eportal/blob/main/README.md#viruses
pub enum Virus {
    SpeedVirus,
    VisionVirus,
}

impl Virus {
    pub const ALL: [Self; 2] = [Self::SpeedVirus, Self::VisionVirus];
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
/// https://github.com/kul-sudo/eportal/blob/main/README.md#skills
pub enum Skill {
    DoNotCompeteWithRelatives,
    DoNotCompeteWithYoungerRelatives,
    AliveWhenArrived,
    ProfitableWhenArrived,
    PrioritizeFasterChasers,
    AvoidNewViruses,
    WillArriveFirst,
    EatCrossesOfMyType,
    AvoidInfectedCrosses,
}

impl Skill {
    pub const ALL: [Self; 9] = [
        Skill::DoNotCompeteWithRelatives,
        Skill::DoNotCompeteWithYoungerRelatives,
        Skill::AliveWhenArrived,
        Skill::ProfitableWhenArrived,
        Skill::PrioritizeFasterChasers,
        Skill::AvoidNewViruses,
        Skill::WillArriveFirst,
        Skill::EatCrossesOfMyType,
        Skill::AvoidInfectedCrosses,
    ];
}

pub type BodyId = Instant;

#[derive(Clone, PartialEq)]
/// https://github.com/kul-sudo/eportal/blob/main/README.md#properties
pub struct Body {
    pub pos:                 Vec2,
    /// Needed for cells: body.pos = body.last_pos after the loop.
    pub last_pos:            Vec2,
    pub energy:              f32,
    pub speed:               f32,
    pub vision_distance:     f32,
    pub eating_strategy:     EatingStrategy,
    pub division_threshold:  f32,
    pub skills:              HashSet<Skill>,
    pub viruses:             HashMap<Virus, f32>,
    pub color:               Color,
    pub status:              Status,
    pub body_type:           u32,
    pub lifespan:            f32,
    initial_speed:           f32,
    initial_vision_distance: f32,
    pub followed_by:         HashMap<BodyId, Self>,
    pub cell:                Cell,
}

#[macro_export]
macro_rules! get_visible {
    ($body:expr, $x:expr, $visible_x:expr) => {
        // Using these for ease of development
        let (a, b) = ($body.pos.x, $body.pos.y);
        let r = $body.vision_distance;
        let (w, h) = (CELLS.cell_width, CELLS.cell_height);
        let (m, n) = (CELLS.columns, CELLS.rows);

        // Get the bottommost, topmost, leftmost, and rightmost rows/columns.
        // If the cell is within the circle or the circle touches the cell, it is
        // within the rectangle around the circle. Some of those cells are unneeded.
        let i_min = ((b - r) / h).floor().max(0.0) as usize;
        let i_max =
        ((b + r) / h).floor().min(n as f32 - 1.0) as usize;
        let j_min = ((a - r) / w).floor().max(0.0) as usize;
        let j_max =
        ((a + r) / w).floor().min(m as f32 - 1.0) as usize;

        // Ditch the unneeded cells
        let Cell {
        i: circle_center_i, ..
        } = CELLS.get_cell_by_pos($body.pos);

        for i in i_min..=i_max {
        let (
        // Get the min/max j we have to care about for i
        j_min_for_i,
        j_max_for_i,
        );

        if i == circle_center_i {
        (j_min_for_i, j_max_for_i) = (j_min, j_max);
        } else {
        let i_for_line =
        if i < circle_center_i { i + 1 } else { i };

        let delta = r
        * (1.0
        - ((i_for_line as f32 * h - b) / r)
        .powi(2))
        .sqrt();
        (j_min_for_i, j_max_for_i) = (
        ((a - delta) / w).floor().max(0.0) as usize,
        ((a + delta) / w).floor().min((m - 1) as f32)
        as usize,
        )
        }

        for j in j_min_for_i..=j_max_for_i {
        // Center of the cell
        let (center_x, center_y) = (
        j as f32 * w + w / 2.0,
        i as f32 * h + h / 2.0,
        );

        // true as usize = 1
        // false as usize = 0
        let (i_delta, j_delta) = (
        (center_y > b) as usize, // If the cell is in the 1st or 2nd quadrant
        (center_x > a) as usize, // If the cell is in the 1st or 4th quadrant
        );

        let fully_covered = (((j + j_delta) as f32) * w
        - a)
        .powi(2)
        + (((i + i_delta) as f32) * h - b).powi(2)
        < r.powi(2);

        for (x_id, x) in
        &$x[i][j]
        {
        if fully_covered
        || $body.pos.distance(x.pos)
        <= $body.vision_distance
        {
        $visible_x.insert(x_id, x);
        }
        }
        }
        }
    }
    }

#[allow(clippy::too_many_arguments)]
impl Body {
    /// https://github.com/kul-sudo/eportal/blob/main/README.md#procreation may be helpful.
    #[inline(always)]
    pub fn new(
        pos: Vec2,
        energy: Option<f32>,
        eating_strategy: EatingStrategy,
        division_threshold: Option<f32>,
        skills: Option<HashSet<Skill>>,
        color: Color,
        body_type: u32,
        viruses: Option<HashMap<Virus, f32>>,
        initial_speed: Option<f32>,
        initial_vision_distance: Option<f32>,
        rng: &mut StdRng,
    ) -> Self {
        let speed = get_with_deviation(
            match initial_speed {
                Some(initial_speed) => initial_speed,
                None => unsafe { AVERAGE_SPEED },
            },
            rng,
        );

        let vision_distance = get_with_deviation(
            match initial_vision_distance {
                Some(initial_vision_distance) => {
                    initial_vision_distance
                }
                None => unsafe { AVERAGE_VISION_DISTANCE },
            },
            rng,
        );

        let mut body = Self {
            pos,
            last_pos: pos,
            energy: match energy {
                Some(energy) => energy / 2.0,
                None => {
                    get_with_deviation(unsafe { AVERAGE_ENERGY }, rng)
                }
            },
            speed,
            initial_speed: speed,
            vision_distance,
            initial_vision_distance: vision_distance,
            eating_strategy,
            division_threshold: get_with_deviation(
                match division_threshold {
                    Some(division_threshold) => division_threshold,
                    None => unsafe { AVERAGE_DIVISION_THRESHOLD },
                },
                rng,
            ),
            skills: match skills {
                Some(mut skills) => {
                    if rng.gen_range(0.0..1.0)
                        <= unsafe { SKILLS_CHANGE_CHANCE }
                    {
                        if random::<bool>() {
                            if let Some(random_skill) =
                                HashSet::from(Skill::ALL)
                                    .difference(&skills)
                                    .collect::<HashSet<_>>()
                                    .iter()
                                    .choose(rng)
                            {
                                skills.insert(**random_skill);
                            }
                        } else if let Some(random_skill) =
                            skills.clone().iter().choose(rng)
                        {
                            skills.remove(random_skill);
                        }
                    }

                    skills
                }
                None => HashSet::with_capacity(Skill::ALL.len()),
            },
            color,
            status: Status::Undefined,
            body_type,
            lifespan: unsafe { LIFESPAN },
            viruses: match viruses {
                Some(viruses) => viruses,
                None => {
                    let mut viruses =
                        HashMap::with_capacity(Virus::ALL.len());

                    if eating_strategy != EatingStrategy::Herbivorous
                    {
                        for virus in Virus::ALL {
                            let virus_chance = match virus {
                                Virus::SpeedVirus => unsafe {
                                    SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE
                                },
                                Virus::VisionVirus => unsafe {
                                    VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE
                                },
                            };

                            if virus_chance == 1.0
                                || rng.gen_range(0.0..1.0)
                                    <= virus_chance
                            {
                                viruses
                                    .entry(virus)
                                    .or_insert(rng.gen_range(
                                    0.0..match virus {
                                        Virus::SpeedVirus => unsafe {
                                            SPEEDVIRUS_HEAL_ENERGY
                                        },
                                        Virus::VisionVirus => unsafe {
                                            VISIONVIRUS_HEAL_ENERGY
                                        },
                                    },
                                ));
                            }
                        }
                    }

                    viruses
                }
            },
            followed_by: HashMap::new(),
            cell: CELLS.get_cell_by_pos(pos),
        };

        // Applying the effect of the viruses
        for virus in unsafe {
            &(*(&body.viruses as *const HashMap<Virus, f32>))
        }
        .keys()
        {
            body.apply_virus(*virus);
        }

        body
    }

    #[inline(always)]
    pub fn wrap(&mut self) {
        if self.last_pos.x >= AREA_SIZE.x {
            self.last_pos.x = MIN_GAP;
        } else if self.last_pos.x <= 0.0 {
            self.last_pos.x = AREA_SIZE.x - MIN_GAP;
        }

        if self.last_pos.y >= AREA_SIZE.y {
            self.last_pos.y = MIN_GAP;
        } else if self.last_pos.y <= 0.0 {
            self.last_pos.y = AREA_SIZE.y - MIN_GAP;
        }
    }

    #[inline(always)]
    pub fn draw(&self) {
        let side_length_half = OBJECT_RADIUS / SQRT_2;

        match self.eating_strategy {
            EatingStrategy::Omnivorous => {
                let side_length = side_length_half * 2.0;
                draw_rectangle(
                    self.pos.x - side_length_half,
                    self.pos.y - side_length_half,
                    side_length,
                    side_length,
                    self.color,
                );
            }
            EatingStrategy::Herbivorous => draw_circle(
                self.pos.x,
                self.pos.y,
                OBJECT_RADIUS,
                self.color,
            ),
            EatingStrategy::Carnivorous => {
                let side_length = side_length_half * 2.0;
                draw_rectangle_ex(
                    self.pos.x,
                    self.pos.y,
                    side_length,
                    side_length,
                    DrawRectangleParams {
                        offset:   vec2(0.5, 0.5),
                        rotation: PI / 4.0,
                        color:    self.color,
                    },
                );
            }
        }

        if !self.viruses.is_empty() {
            draw_circle(self.pos.x, self.pos.y, 5.0, RED)
        }
    }

    #[inline(always)]
    pub fn draw_info(&self) {
        let mut to_display_components = Vec::new();

        if unsafe { SHOW_ENERGY } {
            to_display_components
                .push(format!("energy = {}", self.energy as usize));
        }

        if unsafe { SHOW_DIVISION_THRESHOLD } {
            to_display_components.push(format!(
                "dt = {}",
                self.division_threshold as usize
            ));
        }

        if unsafe { SHOW_BODY_TYPE } {
            to_display_components
                .push(format!("body type = {}", self.body_type));
        }

        if unsafe { SHOW_LIFESPAN } {
            to_display_components.push(format!(
                "lifespan = {}",
                self.lifespan as usize
            ));
        }

        if unsafe { SHOW_SKILLS } {
            to_display_components.push(format!(
                "skills = {:?}",
                self.skills
                    .iter()
                    .map(|skill| *skill as u8)
                    .collect::<Vec<_>>()
            ));
        }

        if unsafe { SHOW_VIRUSES } {
            to_display_components.push(format!(
                "viruses = {:?}",
                self.viruses
                    .keys()
                    .map(|virus| *virus as u8)
                    .collect::<Vec<_>>()
            ));
        }

        if !to_display_components.is_empty() {
            let to_display = to_display_components.join(" | ");
            draw_text(
                &to_display,
                self.pos.x
                    - measure_text(
                        &to_display,
                        None,
                        unsafe { BODY_INFO_FONT_SIZE },
                        1.0,
                    )
                    .width
                        / 2.0,
                self.pos.y - OBJECT_RADIUS - MIN_GAP,
                unsafe { BODY_INFO_FONT_SIZE } as f32,
                WHITE,
            );
        }
    }

    #[inline(always)]
    /// Get the body infected with every virus it doesnn't have yet.
    pub fn get_viruses(&mut self, viruses: &HashMap<Virus, f32>) {
        for virus in viruses.keys() {
            if !self.viruses.contains_key(virus) {
                self.viruses.insert(*virus, 0.0);
                self.apply_virus(*virus);
            }
        }
    }

    #[inline(always)]
    /// Make a virus do its job.
    pub fn apply_virus(&mut self, virus: Virus) {
        match virus {
            Virus::SpeedVirus => {
                self.speed -=
                    self.speed * unsafe { SPEEDVIRUS_SPEED_DECREASE }
            }
            Virus::VisionVirus => {
                self.vision_distance -= self.vision_distance
                    * unsafe { VISIONVIRUS_VISION_DISTANCE_DECREASE }
            }
        };
    }

    #[inline(always)]
    /// Get what needs to be drawn. Needed for performance reasons, because there's no reason to
    /// draw anything beyond the zoom rectangle.
    pub fn get_drawing_strategy(
        &self,
        zoom: &Zoom,
    ) -> DrawingStrategy {
        let mut drawing_strategy = DrawingStrategy::default(); // Everything's false
        let mut target_line = None;

        match zoom.extended_rect.unwrap().contains(self.pos) {
            true => {
                // The body can be partially
                // visible/hidden or completely visible
                drawing_strategy.body = true;
                drawing_strategy.vision_distance = true;
                target_line = Some(true);
            }
            false => {
                drawing_strategy.vision_distance = Circle::new(
                    self.pos.x,
                    self.pos.y,
                    self.vision_distance,
                )
                .overlaps_rect(&zoom.rect.unwrap());

                if let Status::FollowingTarget(_, target_pos, _) =
                    self.status
                {
                    if zoom.rect.unwrap().contains(target_pos) {
                        target_line = Some(true);
                    }
                }
            }
        }

        if target_line.is_none() {
            if let Status::FollowingTarget(_, target_pos, _) =
                self.status
            {
                let mut rectangle_sides = HashMap::with_capacity(
                    RectangleCorner::ALL.len(),
                );
                for corner in RectangleCorner::ALL {
                    let (i, j) = match corner {
                        RectangleCorner::TopRight => (1.0, 1.0),
                        RectangleCorner::TopLeft => (-1.0, 1.0),
                        RectangleCorner::BottomRight => (1.0, -1.0),
                        RectangleCorner::BottomLeft => (-1.0, -1.0),
                    };

                    rectangle_sides.insert(
                        corner,
                        vec2(
                            zoom.center_pos.unwrap().x
                                + i * zoom.rect.unwrap().w / 2.0,
                            zoom.center_pos.unwrap().y
                                + j * zoom.rect.unwrap().h / 2.0,
                        ),
                    );
                }

                target_line = Some(
                    [
                        (
                            RectangleCorner::BottomRight,
                            RectangleCorner::BottomLeft,
                        ),
                        (
                            RectangleCorner::TopRight,
                            RectangleCorner::TopLeft,
                        ),
                        (
                            RectangleCorner::TopRight,
                            RectangleCorner::BottomRight,
                        ),
                        (
                            RectangleCorner::TopLeft,
                            RectangleCorner::BottomLeft,
                        ),
                    ]
                    .iter()
                    .any(|(i, j)| {
                        DrawingStrategy::segments_intersect(
                            self.pos,
                            target_pos,
                            *rectangle_sides.get(i).unwrap(),
                            *rectangle_sides.get(j).unwrap(),
                        )
                    }),
                );
            }
        }

        if let Some(target_line_strategy) = target_line {
            drawing_strategy.target_line = target_line_strategy;
        }

        drawing_strategy
    }

    #[inline(always)]
    /// Heal from the viruses the body has and spend energy on it.
    pub fn handle_viruses(&mut self) {
        for (virus, energy_spent_for_healing) in &mut self.viruses {
            match virus {
                Virus::SpeedVirus => {
                    self.energy = (self.energy
                        - unsafe {
                            SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING
                        })
                    .max(0.0);
                    *energy_spent_for_healing += unsafe {
                        SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING
                    };
                }
                Virus::VisionVirus => {
                    self.energy = (self.energy
                        - unsafe {
                            VISIONVIRUS_ENERGY_SPENT_FOR_HEALING
                        })
                    .max(0.0);
                    *energy_spent_for_healing += unsafe {
                        VISIONVIRUS_ENERGY_SPENT_FOR_HEALING
                    };
                }
            }
        }

        self.viruses.retain(|virus, energy_spent_for_healing| {
            *energy_spent_for_healing
                <= match virus {
                    Virus::SpeedVirus => unsafe {
                        SPEEDVIRUS_HEAL_ENERGY
                    },
                    Virus::VisionVirus => unsafe {
                        VISIONVIRUS_HEAL_ENERGY
                    },
                }
        });
    }

    #[inline(always)]
    /// Handle body-eaters walking and plant-eaters being idle.
    pub fn handle_walking(
        &mut self,
        body_id: &BodyId,
        bodies: &mut [Vec<HashMap<BodyId, Self>>],
        crosses: &mut [Vec<HashMap<CrossId, Cross>>],
        plants: &mut [Vec<HashMap<PlantId, Plant>>],
        rng: &mut StdRng,
    ) {
        if let Status::Walking(pos_deviation) = self.status {
            self.last_pos.x += pos_deviation.x;
            self.last_pos.y += pos_deviation.y;

            self.wrap();
        } else {
            let walking_angle = rng.gen_range(0.0..2.0 * PI);
            let pos_deviation = vec2(
                self.speed * walking_angle.cos(),
                self.speed * walking_angle.sin(),
            );

            self.set_status(
                Status::Walking(pos_deviation),
                body_id,
                bodies,
                crosses,
                plants,
            );
        }
    }

    #[inline(always)]
    /// Handle the energy. The function returns if the body has run out of energy.
    pub fn handle_energy(
        &mut self,
        body_id: &BodyId,
        removed_bodies: &mut HashMap<BodyId, Vec2>,
    ) -> bool {
        // The mass is proportional to the energy; to keep the mass up, energy is spent
        self.energy -= unsafe { ENERGY_SPENT_CONST_FOR_MASS }
            * self.energy
            + unsafe { ENERGY_SPENT_CONST_FOR_SKILLS }
                * self.skills.len() as f32
            + unsafe { ENERGY_SPENT_CONST_FOR_VISION_DISTANCE }
                * self.vision_distance.powi(2)
            + unsafe { ENERGY_SPENT_CONST_FOR_MOVEMENT }
                * self.speed.powi(2)
                * self.energy;

        if self.energy <= 0.0 {
            self.status = Status::Cross;
            removed_bodies.insert(*body_id, self.pos);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn handle_lifespan(&mut self) {
        self.lifespan = (self.lifespan
            - unsafe { CONST_FOR_LIFESPAN }
                * self.speed.powi(2)
                * self.energy)
            .max(0.0)
    }

    #[inline(always)]
    /// Handle procreation and return if one has happened.
    pub fn handle_procreation(
        &mut self,
        body_id: &BodyId,
        new_bodies: &mut HashMap<BodyId, Self>,
        removed_bodies: &mut HashMap<BodyId, Vec2>,
        rng: &mut StdRng,
    ) -> bool {
        if self.energy > self.division_threshold {
            for _ in 0..2 {
                new_bodies.insert(
                    Instant::now(),
                    Body::new(
                        self.pos,
                        Some(self.energy),
                        self.eating_strategy,
                        Some(self.division_threshold),
                        Some(self.skills.clone()),
                        self.color,
                        self.body_type,
                        Some(self.viruses.clone()),
                        Some(self.initial_speed),
                        Some(self.initial_vision_distance),
                        rng,
                    ),
                );
            }

            removed_bodies.insert(*body_id, self.pos);

            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn get_spent_energy(&self, time: f32) -> f32 {
        time * unsafe { ENERGY_SPENT_CONST_FOR_MOVEMENT }
            * self.speed.powi(2)
            * self.energy
            + unsafe { ENERGY_SPENT_CONST_FOR_MASS } * self.energy
            + unsafe { ENERGY_SPENT_CONST_FOR_SKILLS }
                * self.skills.len() as f32
            + unsafe { ENERGY_SPENT_CONST_FOR_VISION_DISTANCE }
                * self.vision_distance.powi(2)
    }

    /// Generate a random position until it suits certain creteria.
    pub fn randomly_spawn_body(
        bodies: &mut [Vec<HashMap<BodyId, Self>>],
        eating_strategy: EatingStrategy,
        body_type: usize,
        rng: &mut StdRng,
    ) {
        let mut pos = Vec2::default();

        // Make sure the position is far enough from the rest of the bodies and the borders of the area
        while {
            pos.x = rng.gen_range(0.0..AREA_SIZE.x);
            pos.y = rng.gen_range(0.0..AREA_SIZE.y);
            (pos.x <= OBJECT_RADIUS + MIN_GAP
                || pos.x >= AREA_SIZE.x - OBJECT_RADIUS - MIN_GAP)
                || (pos.y <= OBJECT_RADIUS + MIN_GAP
                    || pos.y >= AREA_SIZE.y - OBJECT_RADIUS - MIN_GAP)
                || {
                    let Cell { i, j } = CELLS.get_cell_by_pos(pos);
                    bodies[i][j].values().any(|body| {
                        body.last_pos.distance(pos)
                            < OBJECT_RADIUS * 2.0 + MIN_GAP
                    })
                }
        } {}

        // Make sure the color is different enough
        let real_color_gap = COLOR_GAP
            / ((unsafe {
                OMNIVOROUS_N + HERBIVOROUS_N + CARNIVOROUS_N
            } + 3) as f32)
                .powf(1.0 / 3.0);

        let mut color = Color::from_rgba(
            gen_range(COLOR_MIN, COLOR_MAX),
            gen_range(COLOR_MIN, COLOR_MAX),
            gen_range(COLOR_MIN, COLOR_MAX),
            255,
        );

        let green_rgb = Vec3 {
            x: GREEN.r,
            y: GREEN.g,
            z: GREEN.b,
        };

        let red_rgb = Vec3 {
            x: RED.r,
            y: RED.g,
            z: RED.b,
        };

        while bodies.iter().any(|row| {
            row.iter().any(|column| {
                column.values().any(|body| {
                    let current_body_rgb = Vec3 {
                        x: body.color.r,
                        y: body.color.g,
                        z: body.color.b,
                    };
                    current_body_rgb.distance(green_rgb)
                        < real_color_gap
                        || current_body_rgb.distance(red_rgb)
                            < real_color_gap
                        || current_body_rgb.distance(Vec3 {
                            x: color.r,
                            y: color.g,
                            z: color.b,
                        }) < real_color_gap
                })
            })
        }) {
            color = Color::from_rgba(
                gen_range(COLOR_MIN, COLOR_MAX),
                gen_range(COLOR_MIN, COLOR_MAX),
                gen_range(COLOR_MIN, COLOR_MAX),
                255,
            )
        }

        let Cell { i, j } = CELLS.get_cell_by_pos(pos);

        bodies[i][j].insert(
            Instant::now(),
            Body::new(
                pos,
                None,
                eating_strategy,
                None,
                None,
                color,
                body_type as u32,
                None,
                None,
                None,
                rng,
            ),
        );
    }

    pub fn set_status(
        &mut self,
        status: Status,
        body_id: &BodyId,
        bodies: &mut [Vec<HashMap<BodyId, Self>>],
        crosses: &mut [Vec<HashMap<CrossId, Cross>>],
        plants: &mut [Vec<HashMap<PlantId, Plant>>],
    ) {
        Body::followed_by_cleanup(
            body_id,
            &CELLS.get_cell_by_pos(self.pos),
            bodies,
            crosses,
            plants,
            None,
        );
        self.status = status;
    }

    #[inline(always)]
    pub fn find_food<'a>(
        &self,
        body_id: &BodyId,
        bodies: &'a Vec<Vec<HashMap<BodyId, Body>>>,
        plants: &'a Vec<Vec<HashMap<PlantId, Plant>>>,
        crosses: &'a Vec<Vec<HashMap<CrossId, Cross>>>,
        removed_bodies: &HashMap<Instant, Vec2>,
        removed_plants: &HashMap<Instant, Vec2>,
    ) -> Option<FoodInfo<'a>> {
        let mut visible_crosses: HashMap<&CrossId, &Cross> =
            HashMap::new();

        if self.eating_strategy == EatingStrategy::Omnivorous
            || self.eating_strategy == EatingStrategy::Carnivorous
        {
            get_visible!(self, crosses, visible_crosses);
        }

        let closest_cross = visible_crosses
            .iter()
            .filter(|(_, cross)| {
                self.handle_eat_crosses_of_my_type(cross)
                    && self.handle_alive_when_arrived_cross(cross)
                    && self
                        .handle_profitable_when_arrived_cross(cross)
                    && self.handle_avoid_new_viruses_cross(cross)
                    && self.handle_will_arrive_first_cross(
                        body_id, cross,
                    )
                    && self.handle_do_not_compete_with_relatives(
                        body_id,
                        &cross.followed_by,
                    )
                    && self
                        .handle_do_not_compete_with_younger_relatives(
                            body_id,
                            &cross.followed_by,
                        )
            })
            .min_by(|(_, a), (_, b)| {
                self.pos
                    .distance(a.pos)
                    .partial_cmp(&self.pos.distance(b.pos))
                    .unwrap()
            });

        // Find the closest cross
        match closest_cross {
            Some((closest_cross_id, closest_cross)) => {
                return Some(FoodInfo {
                    id:        **closest_cross_id,
                    food_type: ObjectType::Cross,
                    pos:       closest_cross.pos,
                    energy:    closest_cross.energy,
                    viruses:   Some(&closest_cross.viruses),
                })
            }
            None => {
                let mut visible_plants: HashMap<&PlantId, &Plant> =
                    HashMap::new();

                if self.eating_strategy == EatingStrategy::Omnivorous
                    || self.eating_strategy
                        == EatingStrategy::Herbivorous
                {
                    get_visible!(self, plants, visible_plants);
                }

                let filtered_visible_plants = visible_plants
                    .iter()
                    .filter(|(plant_id, plant)| {
                        !removed_plants.contains_key(plant_id)
                        && self.handle_alive_when_arrived_plant(plant)
                        && self.handle_profitable_when_arrived_plant(plant)
                        && self.handle_do_not_compete_with_relatives(
                            body_id,
                            &plant.followed_by
                        )
                        && self.handle_do_not_compete_with_younger_relatives(
                            body_id,
                            &plant.followed_by
                        )
                        && self.handle_will_arrive_first_plant(
                            body_id,
                            plant,
                        )
                    }).collect::<Vec<_>>();

                match self
                    .find_closest_plant(
                        &filtered_visible_plants,
                        PlantKind::Banana,
                    )
                    .or_else(|| {
                        self.find_closest_plant(
                            &filtered_visible_plants,
                            PlantKind::Grass,
                        )
                    }) {
                    Some((closest_plant_id, closest_plant)) => {
                        return Some(FoodInfo {
                            id:        ***closest_plant_id,
                            food_type: ObjectType::Plant,
                            pos:       closest_plant.pos,
                            energy:    closest_plant
                                .get_contained_energy(),
                            viruses:   None,
                        })
                    }
                    None => {
                        // Find the closest plant
                        let mut visible_bodies: HashMap<
                            &BodyId,
                            &Body,
                        > = HashMap::new();

                        if self.eating_strategy
                            == EatingStrategy::Omnivorous
                            || self.eating_strategy
                                == EatingStrategy::Carnivorous
                        {
                            get_visible!(
                                self,
                                bodies,
                                visible_bodies
                            );
                        }

                        let closest_body = visible_bodies
                            .iter()
                            .filter(|(other_body_id, other_body)| {
                                self.body_type != other_body.body_type &&
                                &&body_id != other_body_id
                                && self.pos.distance(other_body.pos)
                                <= self.vision_distance
                                && !removed_bodies.contains_key(other_body_id)
                                && if self.eating_strategy == EatingStrategy::Carnivorous {
                                    self.energy > match other_body.eating_strategy {
                                        EatingStrategy::Carnivorous => other_body.energy,
                                        EatingStrategy::Herbivorous
                                        | EatingStrategy::Omnivorous => other_body.energy * unsafe {
                                            CARNIVOROUS_ENERGY_CONST
                                        }
                                    }
                                } else {
                                    other_body.energy < match other_body.eating_strategy {
                                        EatingStrategy::Carnivorous => self.energy * unsafe { CARNIVOROUS_ENERGY_CONST },
                                        EatingStrategy::Herbivorous
                                        | EatingStrategy::Omnivorous => self.energy
                                    }
                                }
                                && self.handle_alive_when_arrived_body(
                                    other_body,
                                )
                                && self.handle_profitable_when_arrived_body(
                                    other_body,
                                )
                                && self.handle_avoid_new_viruses_body(other_body)
                                && self.handle_will_arrive_first_body(
                                    body_id,
                                    other_body,
                                )
                                && self.handle_do_not_compete_with_relatives(
                                    body_id,
                                    &other_body.followed_by
                                )
                                && self.handle_do_not_compete_with_younger_relatives(
                                    body_id,
                                    &other_body.followed_by,
                                )
                            })
                            .min_by(|(_, a), (_, b)| {
                                self.pos
                                    .distance(a.pos)
                                    .partial_cmp(&self.pos.distance(b.pos))
                                    .unwrap()
                            });

                        // Find the closest body
                        if let Some((closest_body_id, closest_body)) =
                            closest_body
                        {
                            return Some(FoodInfo {
                                id:        **closest_body_id,
                                food_type: ObjectType::Body,
                                pos:       closest_body.pos,
                                energy:    closest_body.energy,
                                viruses:   Some(
                                    &closest_body.viruses,
                                ),
                            });
                        }
                    }
                }
            }
        }

        None
    }

    #[inline(always)]
    pub fn find_closest_plant<'a>(
        &self,
        visible_plants: &'a [(&&PlantId, &&Plant)],
        plant_kind: PlantKind,
    ) -> Option<&'a (&&'a PlantId, &&Plant)> {
        visible_plants
            .iter()
            .filter(|(_, plant)| plant.kind == plant_kind)
            .min_by(|(_, a), (_, b)| {
                self.pos
                    .distance(a.pos)
                    .partial_cmp(&self.pos.distance(b.pos))
                    .unwrap()
            })
    }

    #[inline(always)]
    pub fn handle_profitable_when_arrived_body(
        &self,
        other_body: &Body,
    ) -> bool {
        if self.skills.contains(&Skill::ProfitableWhenArrived) {
            let divisor = self.speed - other_body.speed;

            if divisor <= 0.0 {
                return false;
            }

            self.get_spent_energy(
                self.pos.distance(other_body.pos) / divisor,
            ) < other_body.energy
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_profitable_when_arrived_plant(
        &self,
        plant: &Plant,
    ) -> bool {
        if self.skills.contains(&Skill::ProfitableWhenArrived) {
            self.get_spent_energy(
                self.pos.distance(plant.pos) / self.speed,
            ) < plant.get_contained_energy()
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_profitable_when_arrived_cross(
        &self,
        cross: &Cross,
    ) -> bool {
        if self.skills.contains(&Skill::ProfitableWhenArrived) {
            self.get_spent_energy(
                self.pos.distance(cross.pos) / self.speed,
            ) < cross.energy
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_alive_when_arrived_cross(
        &self,
        cross: &Cross,
    ) -> bool {
        if self.skills.contains(&Skill::AliveWhenArrived) {
            self.energy
                - self.get_spent_energy(
                    self.pos.distance(cross.pos) / self.speed,
                )
                > unsafe { MIN_ENERGY }
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_alive_when_arrived_body(
        &self,
        other_body: &Self,
    ) -> bool {
        if self.skills.contains(&Skill::AliveWhenArrived) {
            let divisor = self.speed - other_body.speed;

            if divisor <= 0.0 {
                return false;
            }

            self.energy
                - self.get_spent_energy(
                    self.pos.distance(other_body.pos) / divisor,
                )
                > unsafe { MIN_ENERGY }
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_alive_when_arrived_plant(
        &self,
        plant: &Plant,
    ) -> bool {
        if self.skills.contains(&Skill::AliveWhenArrived) {
            self.energy
                - self.get_spent_energy(
                    self.pos.distance(plant.pos) / self.speed,
                )
                > unsafe { MIN_ENERGY }
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_avoid_new_viruses_cross(
        &self,
        cross: &Cross,
    ) -> bool {
        if self.skills.contains(&Skill::AvoidNewViruses) {
            cross
                .viruses
                .keys()
                .all(|virus| self.viruses.contains_key(virus))
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_avoid_new_viruses_body(
        &self,
        other_body: &Self,
    ) -> bool {
        if self.skills.contains(&Skill::AvoidNewViruses) {
            other_body
                .viruses
                .keys()
                .all(|virus| self.viruses.contains_key(virus))
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_do_not_compete_with_relatives(
        &self,
        body_id: &BodyId,
        followed_by: &HashMap<BodyId, Self>,
    ) -> bool {
        if self.skills.contains(&Skill::DoNotCompeteWithRelatives) {
            followed_by.iter().all(|(other_body_id, other_body)| {
                other_body_id == body_id
                    || other_body.body_type != self.body_type
            })
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_do_not_compete_with_younger_relatives(
        &self,
        body_id: &BodyId,
        followed_by: &HashMap<BodyId, Self>,
    ) -> bool {
        if self
            .skills
            .contains(&Skill::DoNotCompeteWithYoungerRelatives)
        {
            followed_by.iter().all(|(other_body_id, other_body)| {
                other_body_id == body_id
                    || if other_body.body_type == self.body_type {
                        other_body.lifespan < self.lifespan
                    } else {
                        true
                    }
            })
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_will_arrive_first_cross(
        &self,
        body_id: &BodyId,
        cross: &Cross,
    ) -> bool {
        if self.skills.contains(&Skill::WillArriveFirst) {
            let time = self.pos.distance(cross.pos) / self.speed;

            cross.followed_by.iter().all(|(chaser_id, chaser)| {
                chaser_id == body_id
                    || time
                        < chaser.pos.distance(cross.pos)
                            / chaser.speed
            })
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_will_arrive_first_body(
        &self,
        body_id: &BodyId,
        other_body: &Self,
    ) -> bool {
        if self.skills.contains(&Skill::WillArriveFirst) {
            let delta = self.speed - other_body.speed;
            if delta <= 0.0 {
                return false;
            }

            let time = self.pos.distance(other_body.pos) / delta;
            other_body.followed_by.iter().all(
                |(chaser_id, chaser)| {
                    chaser_id == body_id || {
                        let chaser_delta =
                            chaser.speed - other_body.speed;

                        chaser_delta > 0.0
                            && time
                                < chaser.pos.distance(other_body.pos)
                                    / chaser_delta
                    }
                },
            )
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_will_arrive_first_plant(
        &self,
        body_id: &BodyId,
        plant: &Plant,
    ) -> bool {
        if self.skills.contains(&Skill::WillArriveFirst) {
            let time = self.pos.distance(plant.pos) / self.speed;

            plant.followed_by.iter().all(|(chaser_id, chaser)| {
                chaser_id == body_id
                    || time
                        < chaser.pos.distance(plant.pos)
                            / chaser.speed
            })
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_eat_crosses_of_my_type(
        &self,
        cross: &Cross,
    ) -> bool {
        self.body_type != cross.body_type
            || self.skills.contains(&Skill::EatCrossesOfMyType)
    }

    #[inline(always)]
    pub fn followed_by_cleanup(
        body_id: &BodyId,
        body_cell: &Cell,
        bodies: &mut [Vec<HashMap<BodyId, Self>>],
        crosses: &mut [Vec<HashMap<CrossId, Cross>>],
        plants: &mut [Vec<HashMap<PlantId, Plant>>],
        food: Option<&FoodInfo>,
    ) {
        if let Status::FollowingTarget(
            target_id,
            target_pos,
            target_type,
        ) = bodies[body_cell.i][body_cell.j]
            .get(body_id)
            .unwrap()
            .status
        {
            if food.is_some_and(|food| food.id == target_id) {
                return;
            }

            let Cell { i, j } = CELLS.get_cell_by_pos(target_pos);
            match target_type {
                ObjectType::Body => {
                    if let Some(target_body) =
                        bodies[i][j].get_mut(&target_id)
                    {
                        target_body.followed_by.remove(body_id);
                    }
                }
                ObjectType::Cross => {
                    if let Some(target_cross) =
                        crosses[i][j].get_mut(&target_id)
                    {
                        target_cross.followed_by.remove(body_id);
                    }
                }
                ObjectType::Plant => {
                    if let Some(target_plant) =
                        plants[i][j].get_mut(&target_id)
                    {
                        target_plant.followed_by.remove(body_id);
                    }
                }
            }
        }
    }
}
