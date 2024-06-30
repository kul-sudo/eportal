use crate::body::Skill;
use crate::constants::*;
use crate::user_constants::*;
use crate::Virus;
use crate::{TOTAL_SKILLS_COUNT, VIRUSES_COUNT};
use serde_derive::Deserialize;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::mem::variant_count;
use std::process::exit;
use toml::from_str;

#[derive(Deserialize)]
struct BodyField {
    bodies_n: usize,
    passive_chance: f32,
    average_energy: f32,
    average_speed: f32,
    average_division_threshold: f32,
    average_vision_distance: f32,
    skills_change_chance: f32,
    deviation: f32,
    lifespan: f32,
    min_energy: f32,
    cross_lifespan: u64,
}

#[derive(Deserialize)]
struct PlantField {
    plants_density: f32,
}

#[derive(Deserialize)]
struct EnergyField {
    energy_spent_const_for_mass: f32,
    energy_spent_const_for_skills: f32,
    energy_spent_const_for_vision_distance: f32,
    energy_spent_const_for_movement: f32,
    const_for_lifespan: f32,
}

#[derive(Deserialize)]
struct VirusesField {
    speedvirus_first_generation_infection_chance: f32,
    speedvirus_speed_decrease: f32,
    speedvirus_energy_spent_for_healing: f32,
    speedvirus_heal_energy: f32,

    visionvirus_first_generation_infection_chance: f32,
    visionvirus_vision_distance_decrease: f32,
    visionvirus_energy_spent_for_healing: f32,
    visionvirus_heal_energy: f32,
}

#[derive(Deserialize)]
pub struct UIField {
    body_info_font_size: u16,

    show_energy: bool,
    show_division_threshold: bool,
    show_body_type: bool,
    show_lifespan: bool,
    show_skills: bool,
    show_viruses: bool,
}

#[derive(Deserialize)]
struct Data {
    body: BodyField,
    plants: PlantField,
    energy: EnergyField,
    viruses: VirusesField,
    ui: UIField,
}

pub fn config_setup() {
    let contents = match read_to_string(CONFIG_FILE_NAME) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("The config file hasn't been found.");
            exit(1);
        }
    };

    let config: Data = match from_str(&contents) {
        Ok(config) => config,
        Err(_) => {
            eprintln!("The file style isn't correct.");
            exit(1);
        }
    };

    let body = config.body;
    let plants = config.plants;
    let energy = config.energy;
    let viruses = config.viruses;
    let ui = config.ui;

    unsafe {
        // Body-related
        BODIES_N = body.bodies_n;
        PASSIVE_CHANCE = body.passive_chance;
        AVERAGE_ENERGY = body.average_energy;
        AVERAGE_SPEED = body.average_speed;
        AVERAGE_DIVISION_THRESHOLD = body.average_division_threshold;
        AVERAGE_VISION_DISTANCE = body.average_vision_distance;

        SKILLS_CHANGE_CHANCE = body.skills_change_chance;
        DEVIATION = body.deviation;
        LIFESPAN = body.lifespan;
        MIN_ENERGY = body.min_energy;
        CROSS_LIFESPAN = body.cross_lifespan;

        // Plants-related
        PLANTS_DENSITY = plants.plants_density;

        // Virus-related
        SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE =
            viruses.speedvirus_first_generation_infection_chance;
        SPEEDVIRUS_SPEED_DECREASE = viruses.speedvirus_speed_decrease;
        SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING = viruses.speedvirus_energy_spent_for_healing;
        SPEEDVIRUS_HEAL_ENERGY = viruses.speedvirus_heal_energy;

        VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE =
            viruses.visionvirus_first_generation_infection_chance;
        VISIONVIRUS_VISION_DISTANCE_DECREASE = viruses.visionvirus_vision_distance_decrease;
        VISIONVIRUS_ENERGY_SPENT_FOR_HEALING = viruses.visionvirus_energy_spent_for_healing;
        VISIONVIRUS_HEAL_ENERGY = viruses.visionvirus_heal_energy;

        // Energy-related
        ENERGY_SPENT_CONST_FOR_MASS = energy.energy_spent_const_for_mass;
        ENERGY_SPENT_CONST_FOR_SKILLS = energy.energy_spent_const_for_skills;
        ENERGY_SPENT_CONST_FOR_VISION_DISTANCE = energy.energy_spent_const_for_vision_distance;
        ENERGY_SPENT_CONST_FOR_MOVEMENT = energy.energy_spent_const_for_movement;
        CONST_FOR_LIFESPAN = energy.const_for_lifespan;

        // UI-related
        BODY_INFO_FONT_SIZE = ui.body_info_font_size;

        SHOW_ENERGY = ui.show_energy;
        SHOW_DIVISION_THRESHOLD = ui.show_division_threshold;
        SHOW_BODY_TYPE = ui.show_body_type;
        SHOW_LIFESPAN = ui.show_lifespan;
        SHOW_SKILLS = ui.show_skills;
        SHOW_VIRUSES = ui.show_viruses;
    };
}

pub fn enum_consts() -> (HashSet<usize>, HashSet<usize>) {
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