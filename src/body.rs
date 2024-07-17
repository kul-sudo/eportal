use crate::user_constants::*;
use crate::{TOTAL_SKILLS_COUNT, VIRUSES_COUNT};
use macroquad::prelude::{
    draw_circle, draw_line, draw_rectangle, draw_text, measure_text,
    rand::gen_range, vec2, Circle, Color, Vec2, Vec3, GREEN, RED,
    WHITE,
};
use rand::random;
use rand::{rngs::StdRng, seq::IteratorRandom, Rng};
use std::collections::HashSet;
use std::f32::consts::PI;
use std::mem::transmute;
use std::{
    collections::HashMap, f32::consts::SQRT_2, intrinsics::unlikely,
    time::Instant,
};

use crate::{
    constants::*,
    get_with_deviation,
    plant::Plant,
    smart_drawing::{DrawingStrategy, RectangleCorner},
    zoom::Zoom,
    UI_SHOW_PROPERTIES_N,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    FollowingTarget((Instant, Vec2)),
    EscapingBody((Instant, u16)),
    Dead(Instant),
    Walking(Vec2),
    Idle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EatingStrategy {
    /// When a body sees no food, it stands still.
    Passive,
    /// When a body sees no food, it walks in random directions, hoping to find it.
    Active,
}

#[allow(dead_code)]
#[repr(usize)]
#[derive(PartialEq, Hash)]
/// https://github.com/kul-sudo/eportal/blob/main/README.md#viruses
pub enum Virus {
    SpeedVirus,
    VisionVirus,
}

/// https://github.com/kul-sudo/eportal/blob/main/README.md#skills
pub enum Skill {
    DoNotCompeteWithRelatives,
    AliveWhenArrived,
    ProfitableWhenArrived,
    PrioritizeFasterChasers,
    AvoidNewViruses,
    WillArriveFirst,
    EatCrossesOfMyType,
    AvoidInfectedCrosses,
}

#[derive(Clone, PartialEq)]
/// https://github.com/kul-sudo/eportal/blob/main/README.md#properties
pub struct Body {
    pub pos:                     Vec2,
    pub energy:                  f32,
    pub speed:                   f32,
    pub vision_distance:         f32,
    pub eating_strategy:         EatingStrategy,
    pub division_threshold:      f32,
    pub skills:                  HashSet<usize>,
    pub viruses:                 HashMap<usize, f32>,
    pub color:                   Color,
    pub status:                  Status,
    pub body_type:               u16,
    pub lifespan:                f32,
    pub initial_speed:           f32,
    pub initial_vision_distance: f32,
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
        skills: Option<HashSet<usize>>,
        color: Color,
        body_type: u16,
        viruses: Option<HashMap<usize, f32>>,
        initial_speed: Option<f32>,
        initial_vision_distance: Option<f32>,
        all_skills: &HashSet<usize>,
        all_viruses: &HashSet<usize>,
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

        let mut body = Body {
            pos,
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
                            if let Some(random_skill) = all_skills
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
                None => HashSet::with_capacity(unsafe {
                    TOTAL_SKILLS_COUNT
                }),
            },
            color,
            status: Status::Idle,
            body_type,
            lifespan: unsafe { LIFESPAN },
            viruses: match viruses {
                Some(viruses) => viruses,
                None => {
                    let mut viruses =
                        HashMap::with_capacity(unsafe {
                            VIRUSES_COUNT
                        });
                    for virus in all_viruses {
                        let virus_cast = unsafe {
                            transmute::<usize, Virus>(*virus)
                        };
                        if rng.gen_range(0.0..1.0)
                            <= match virus_cast {
                                Virus::SpeedVirus => unsafe {
                                    SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE
                                },
                                Virus::VisionVirus => unsafe {
                                    VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE
                                },
                            }
                        {
                            viruses.insert(
                                *virus,
                                rng.gen_range(
                                    0.0..match virus_cast {
                                        Virus::SpeedVirus => unsafe {
                                            SPEEDVIRUS_HEAL_ENERGY
                                        },
                                        Virus::VisionVirus => unsafe {
                                            VISIONVIRUS_HEAL_ENERGY
                                        },
                                    },
                                ), // Assuming the evolution
                                   // theoretically starts before it starts being shown
                            );
                        }
                    }

                    viruses
                }
            },
        };

        // Applying the effect of the viruses
        for virus in body.viruses.clone().keys() {
            body.apply_virus(virus);
        }

        body
    }

    #[inline(always)]
    pub fn is_alive(&self) -> bool {
        !matches!(self.status, Status::Dead(..))
    }

    pub fn wrap(&mut self, area_size: &Vec2) {
        if self.pos.x >= area_size.x {
            self.pos.x = MIN_GAP;
        } else if self.pos.x <= 0.0 {
            self.pos.x = area_size.x - MIN_GAP;
        }

        if self.pos.y >= area_size.y {
            self.pos.y = MIN_GAP;
        } else if self.pos.y <= 0.0 {
            self.pos.y = area_size.y - MIN_GAP;
        }
    }

    #[inline(always)]
    pub fn draw(&self, zoom: &Zoom, zoom_mode: bool) {
        if zoom_mode {
            if let Some(extended_rect) = zoom.extended_rect {
                if self.pos.distance(extended_rect.center())
                    >= self.vision_distance
                        + zoom.diagonal_extended_rect / 2.0
                {
                    return;
                }
            }
        }

        let side_length_half = OBJECT_RADIUS / SQRT_2;

        if self.is_alive() {
            match self.eating_strategy {
                EatingStrategy::Active => {
                    let side_length = side_length_half * 2.0;
                    draw_rectangle(
                        self.pos.x - side_length_half,
                        self.pos.y - side_length_half,
                        side_length,
                        side_length,
                        self.color,
                    );
                }
                EatingStrategy::Passive => draw_circle(
                    self.pos.x,
                    self.pos.y,
                    OBJECT_RADIUS,
                    self.color,
                ),
            }
        } else {
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
            )
        }

        if !self.viruses.is_empty() {
            draw_circle(self.pos.x, self.pos.y, 5.0, RED)
        }
    }

    pub fn draw_info(&self) {
        let mut to_display_components =
            Vec::with_capacity(unsafe { UI_SHOW_PROPERTIES_N });

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
            to_display_components
                .push(format!("skills = {:?}", self.skills));
        }

        if unsafe { SHOW_VIRUSES } {
            to_display_components
                .push(format!("viruses = {:?}", self.viruses.keys()));
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
    pub fn get_viruses(&mut self, viruses: &HashMap<usize, f32>) {
        for virus in viruses.keys() {
            if !self.viruses.contains_key(virus) {
                self.viruses.insert(*virus, 0.0);
                self.apply_virus(virus);
            }
        }
    }

    #[inline(always)]
    /// Make a virus do its job.
    pub fn apply_virus(&mut self, virus: &usize) {
        match unsafe { transmute::<usize, Virus>(*virus) } {
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
        let mut drawing_strategy = DrawingStrategy::default();
        let mut target_line = None; // It hasn't been decided yet whether it's needed to draw a
                                    // line

        let mut rectangle_corners = HashMap::with_capacity(4);
        for corner in [
            RectangleCorner::TopRight,
            RectangleCorner::TopLeft,
            RectangleCorner::BottomRight,
            RectangleCorner::BottomLeft,
        ] {
            let (i, j) = match corner {
                RectangleCorner::TopRight => (1.0, 1.0),
                RectangleCorner::TopLeft => (-1.0, 1.0),
                RectangleCorner::BottomRight => (1.0, -1.0),
                RectangleCorner::BottomLeft => (-1.0, -1.0),
            };

            rectangle_corners.insert(
                corner,
                vec2(
                    zoom.center_pos.unwrap().x
                        + i * zoom.rect.unwrap().w / 2.0,
                    zoom.center_pos.unwrap().y
                        + j * zoom.rect.unwrap().h / 2.0,
                ),
            );
        }

        // Step 1
        if zoom.extended_rect.unwrap().contains(self.pos) {
            // The body can be partially
            // visible/hidden or completely visible
            drawing_strategy.body = true;

            if !self.is_alive() {
                return drawing_strategy;
            }

            drawing_strategy.vision_distance = true;

            if let Status::FollowingTarget((_, target_pos)) =
                self.status
            {
                // If it isn't inside the rectangle, it's determined later on
                if zoom.rect.unwrap().contains(target_pos) {
                    target_line = Some(true);
                }
            }
        } else {
            if !self.is_alive() {
                return drawing_strategy;
            }

            drawing_strategy.body = false;
            drawing_strategy.vision_distance = Circle::new(
                self.pos.x,
                self.pos.y,
                self.vision_distance,
            )
            .overlaps_rect(&zoom.rect.unwrap());
        }

        // Step 2
        if let Status::FollowingTarget((_, target_pos)) = self.status
        {
            if !drawing_strategy.body {
                // It's handled here if it's unneeded to draw the target line
                if drawing_strategy.vision_distance {
                    target_line = None;
                } else {
                    target_line = if unlikely(
                        self.vision_distance > zoom.diagonal_rect,
                    )
                    // It is very unlikely for the vision distance to be big enough to cover the
                    // whole diagonal of the zoom area
                    {
                        None
                    } else {
                        Some(false)
                    }
                }
            }

            if target_line.is_none() {
                target_line = Some(false);

                for (i, j) in [
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
                ] {
                    if DrawingStrategy::segments_intersect(
                        &self.pos,
                        &target_pos,
                        rectangle_corners.get(&i).unwrap(),
                        rectangle_corners.get(&j).unwrap(),
                    ) {
                        target_line = Some(true);
                        break;
                    }
                }
            }

            drawing_strategy.target_line = target_line.unwrap();
        } else {
            drawing_strategy.target_line = false;
        }

        drawing_strategy
    }

    #[inline(always)]
    /// Heal from the viruses the body has and spend energy on it.
    pub fn handle_viruses(&mut self) {
        for (virus, energy_spent_for_healing) in &mut self.viruses {
            match unsafe { transmute::<usize, Virus>(*virus) } {
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
                <= match unsafe { transmute::<usize, Virus>(*virus) }
                {
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
    pub fn handle_walking_idle(
        &mut self,
        area_size: &Vec2,
        rng: &mut StdRng,
    ) {
        match self.eating_strategy {
            EatingStrategy::Active => {
                if !matches!(self.status, Status::Walking(..)) {
                    let walking_angle: f32 =
                        rng.gen_range(0.0..2.0 * PI);
                    let pos_deviation = vec2(
                        self.speed * walking_angle.cos(),
                        self.speed * walking_angle.sin(),
                    );

                    self.status = Status::Walking(pos_deviation);
                }

                if let Status::Walking(pos_deviation) = self.status {
                    self.pos.x += pos_deviation.x;
                    self.pos.y += pos_deviation.y;
                }

                self.wrap(area_size);
            }
            EatingStrategy::Passive => self.status = Status::Idle,
        }
    }

    #[inline(always)]
    /// Handle the energy. The function returns if the body has run out of energy.
    pub fn handle_energy(
        &mut self,
        body_id: &Instant,
        removed_bodies: &mut HashSet<Instant>,
    ) -> bool {
        // The mass is proportional to the energy; to keep the mass up, energy is spent
        self.energy -= unsafe { ENERGY_SPENT_CONST_FOR_MASS }
            * self.energy
            + unsafe { ENERGY_SPENT_CONST_FOR_SKILLS }
                * self.skills.len() as f32
            + unsafe { ENERGY_SPENT_CONST_FOR_VISION_DISTANCE }
                * self.vision_distance.powi(2);

        if self.status != Status::Idle {
            self.energy -= unsafe { ENERGY_SPENT_CONST_FOR_MOVEMENT }
                * self.speed.powi(2)
                * self.energy;
        }

        if self.energy <= 0.0 {
            removed_bodies.insert(*body_id);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn handle_lifespan(&mut self) {
        if self.status != Status::Idle {
            self.lifespan = (self.lifespan
                - unsafe { CONST_FOR_LIFESPAN }
                    * self.speed.powi(2)
                    * self.energy)
                .max(0.0)
        }
    }

    #[inline(always)]
    /// Handle procreation and return if one has happened.
    pub fn handle_procreation(
        &mut self,
        body_id: &Instant,
        new_bodies: &mut HashMap<Instant, Body>,
        removed_bodies: &mut HashSet<Instant>,
        all_skills: &HashSet<usize>,
        all_viruses: &HashSet<usize>,
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
                        all_skills,
                        all_viruses,
                        rng,
                    ),
                );
            }
            removed_bodies.insert(*body_id);

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
        bodies: &mut HashMap<Instant, Body>,
        area_size: &Vec2,
        eating_strategy: EatingStrategy,
        body_type: usize,
        all_skills: &HashSet<usize>,
        all_viruses: &HashSet<usize>,
        rng: &mut StdRng,
    ) {
        let mut pos = Vec2::default();

        // Make sure the position is far enough from the rest of the bodies and the borders of the area
        while {
            pos.x = rng.gen_range(0.0..area_size.x);
            pos.y = rng.gen_range(0.0..area_size.y);
            (pos.x <= OBJECT_RADIUS + MIN_GAP
                || pos.x >= area_size.x - OBJECT_RADIUS - MIN_GAP)
                || (pos.y <= OBJECT_RADIUS + MIN_GAP
                    || pos.y >= area_size.y - OBJECT_RADIUS - MIN_GAP)
                || bodies.values().any(|body| {
                    body.pos.distance(pos)
                        < OBJECT_RADIUS * 2.0 + MIN_GAP
                })
        } {}

        // Make sure the color is different enough
        let real_color_gap = COLOR_GAP
            / ((unsafe { BODIES_N } + 2) as f32).powf(1.0 / 3.0);

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

        while bodies.values().any(|body| {
            let current_body_rgb = Vec3 {
                x: body.color.r,
                y: body.color.g,
                z: body.color.b,
            };
            current_body_rgb.distance(green_rgb) < real_color_gap
                || current_body_rgb.distance(red_rgb) < real_color_gap
                || current_body_rgb.distance(Vec3 {
                    x: color.r,
                    y: color.g,
                    z: color.b,
                }) < real_color_gap
        }) {
            color = Color::from_rgba(
                gen_range(COLOR_MIN, COLOR_MAX),
                gen_range(COLOR_MIN, COLOR_MAX),
                gen_range(COLOR_MIN, COLOR_MAX),
                255,
            )
        }

        bodies.insert(
            Instant::now(),
            Body::new(
                pos,
                None,
                eating_strategy,
                None,
                None,
                color,
                body_type as u16,
                None,
                None,
                None,
                all_skills,
                all_viruses,
                rng,
            ),
        );
    }

    #[inline(always)]
    pub fn handle_profitable_when_arrived_body(
        &self,
        other_body: &Body,
        target_immovable: bool,
    ) -> bool {
        if self
            .skills
            .contains(&(Skill::ProfitableWhenArrived as usize))
        {
            let divisor = if target_immovable {
                self.speed
            } else {
                self.speed - other_body.speed
            };

            if divisor <= 0.0 {
                return false;
            }

            let time = self.pos.distance(other_body.pos) / divisor;

            self.get_spent_energy(time) < other_body.energy
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_profitable_when_arrived_plant(
        &self,
        plant: &Plant,
    ) -> bool {
        if self
            .skills
            .contains(&(Skill::ProfitableWhenArrived as usize))
        {
            let time = self.pos.distance(plant.pos) / self.speed;

            self.get_spent_energy(time) < plant.get_contained_energy()
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_alive_when_arrived_body(
        &self,
        other_body: &Body,
        target_immovable: bool,
    ) -> bool {
        if self.skills.contains(&(Skill::AliveWhenArrived as usize)) {
            let divisor = if target_immovable {
                self.speed
            } else {
                self.speed - other_body.speed
            };

            if divisor <= 0.0 {
                return false;
            }

            let time = self.pos.distance(other_body.pos) / divisor;

            self.energy - self.get_spent_energy(time)
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
        if self.skills.contains(&(Skill::AliveWhenArrived as usize)) {
            let time = self.pos.distance(plant.pos) / self.speed;

            self.energy - self.get_spent_energy(time)
                > unsafe { MIN_ENERGY }
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_avoid_new_viruses(
        &self,
        other_body: &Body,
    ) -> bool {
        let is_alive = self.is_alive();

        if (is_alive
            && self
                .skills
                .contains(&(Skill::AvoidNewViruses as usize)))
            || (!is_alive
                && self.skills.contains(
                    &(Skill::AvoidInfectedCrosses as usize),
                ))
        {
            other_body
                .viruses
                .keys()
                .all(|virus| self.viruses.contains_key(virus))
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_do_not_complete_with_relatives(
        &self,
        id: &Instant,
        pos: &Vec2,
        bodies_shot_for_statuses: &HashMap<Instant, Body>,
        bodies_within_vision_distance_of_my_type: &[&(
            &Instant,
            &Body,
        )],
    ) -> bool {
        if self
            .skills
            .contains(&(Skill::DoNotCompeteWithRelatives as usize))
        {
            bodies_within_vision_distance_of_my_type.iter().all(
                |(other_body_id, _)| {
                    bodies_shot_for_statuses
                        .get(other_body_id)
                        .unwrap()
                        .status
                        != Status::FollowingTarget((*id, *pos))
                },
            )
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_will_arive_first_body(
        &self,
        other_body_id: &Instant,
        other_body: &Body,
        bodies_within_vision_distance: &[(&Instant, &Body)],
    ) -> bool {
        let mut other_body_speed = other_body.speed;

        if self.is_alive() {
            other_body_speed = 0.0;
        }

        if self.skills.contains(&(Skill::WillArriveFirst as usize)) {
            let delta = self.speed - other_body_speed;
            if delta <= 0.0 {
                return false;
            }

            let time = self.pos.distance(other_body.pos) / delta;

            bodies_within_vision_distance.iter().any(
                |(_, other_body_1)| {
                    if let Status::FollowingTarget((target_id, _)) =
                        other_body_1.status
                    {
                        &target_id == other_body_id && {
                            let delta_1 =
                                other_body_1.speed - other_body_speed;
                            if delta_1 <= 0.0 {
                                return false;
                            }
                            let time_1 = other_body_1
                                .pos
                                .distance(other_body.pos)
                                / delta_1;

                            time > time_1
                        }
                    } else {
                        false
                    }
                },
            )
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_will_arive_first_plant(
        &self,
        plant_id: &Instant,
        plant: &Plant,
        bodies_within_vision_distance: &[(&Instant, &Body)],
    ) -> bool {
        if self.skills.contains(&(Skill::WillArriveFirst as usize)) {
            let time = self.pos.distance(plant.pos) / self.speed;

            bodies_within_vision_distance.iter().any(
                |(_, other_body_1)| {
                    if let Status::FollowingTarget((target_id, _)) =
                        other_body_1.status
                    {
                        &target_id == plant_id && {
                            let time_1 =
                                other_body_1.pos.distance(plant.pos)
                                    / other_body_1.speed;

                            time > time_1
                        }
                    } else {
                        false
                    }
                },
            )
        } else {
            true
        }
    }

    #[inline(always)]
    pub fn handle_eat_crosses_of_my_type(
        &self,
        cross: &Body,
    ) -> bool {
        self.body_type != cross.body_type
            || self
                .skills
                .contains(&(Skill::EatCrossesOfMyType as usize))
    }
}
