use crate::constants::*;
use macroquad::prelude::*;
use serde_derive::Deserialize;
use std::fs::read_to_string;
use std::process::exit;
use toml::from_str;

// Average spawn attributes
pub static mut BODIES_N: usize = 0;
pub static mut PASSIVE_CHANCE: f32 = 0.0;

pub static mut AVERAGE_ENERGY: f32 = 0.0;
pub static mut AVERAGE_SPEED: f32 = 0.0;
pub static mut AVERAGE_DIVISION_THRESHOLD: f32 = 0.0;
pub static mut AVERAGE_VISION_DISTANCE: f32 = 0.0;

pub static mut SKILLS_CHANGE_CHANCE: f32 = 0.0;
pub static mut PLANTS_DENSITY: f32 = 0.0;

pub static mut DEVIATION: f32 = 0.0;
pub static mut LIFESPAN: f32 = 0.0;
pub static mut MIN_ENERGY: f32 = 0.0;

pub static mut PLANT_SPAWN_CHANCE: f32 = 0.0;
pub static mut PLANT_DIE_CHANCE: f32 = 0.0;

// Death
pub static mut CROSS_LIFESPAN: u64 = 0;

// Spending energy
pub static mut ENERGY_SPENT_CONST_FOR_MASS: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_SKILLS: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_VISION_DISTANCE: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_MOVEMENT: f32 = 0.0;
pub static mut CONST_FOR_LIFESPAN: f32 = 0.0;

// SpeedVirus
pub static mut SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE: f32 =
    0.0;
pub static mut SPEEDVIRUS_SPEED_DECREASE: f32 = 0.0;
pub static mut SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING: f32 = 0.0;
pub static mut SPEEDVIRUS_HEAL_ENERGY: f32 = 0.0;

// VisionVirus
pub static mut VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE: f32 =
    0.0;
pub static mut VISIONVIRUS_VISION_DISTANCE_DECREASE: f32 = 0.0;
pub static mut VISIONVIRUS_ENERGY_SPENT_FOR_HEALING: f32 = 0.0;
pub static mut VISIONVIRUS_HEAL_ENERGY: f32 = 0.0;

// Conditions
pub static mut FEWER_PLANTS_CHANCE: f32 = 0.0;
pub static mut MORE_PLANTS_CHANCE: f32 = 0.0;

// UI
pub static mut BODY_INFO_FONT_SIZE: u16 = 0;
pub static mut SHOW_FPS: bool = false;

pub static mut SHOW_ENERGY: bool = false;
pub static mut SHOW_DIVISION_THRESHOLD: bool = false;
pub static mut SHOW_BODY_TYPE: bool = false;
pub static mut SHOW_LIFESPAN: bool = false;
pub static mut SHOW_SKILLS: bool = false;
pub static mut SHOW_VIRUSES: bool = false;

#[derive(Deserialize)]
struct BodyField {
    bodies_n:                   usize,
    passive_chance:             f32,
    average_energy:             f32,
    average_speed:              f32,
    average_division_threshold: f32,
    average_vision_distance:    f32,
    skills_change_chance:       f32,
    deviation:                  f32,
    lifespan:                   f32,
    min_energy:                 f32,
    cross_lifespan:             u64,
    const_for_lifespan:         f32,
}

#[derive(Deserialize)]
struct PlantField {
    plants_density:     f32,
    plant_spawn_chance: f32,
    plant_die_chance:   f32,
}

#[derive(Deserialize)]
struct EnergyField {
    energy_spent_const_for_mass:            f32,
    energy_spent_const_for_skills:          f32,
    energy_spent_const_for_vision_distance: f32,
    energy_spent_const_for_movement:        f32,
}

#[derive(Deserialize)]
struct VirusesField {
    speedvirus_first_generation_infection_chance: f32,
    speedvirus_speed_decrease:                    f32,
    speedvirus_energy_spent_for_healing:          f32,
    speedvirus_heal_energy:                       f32,

    visionvirus_first_generation_infection_chance: f32,
    visionvirus_vision_distance_decrease:          f32,
    visionvirus_energy_spent_for_healing:          f32,
    visionvirus_heal_energy:                       f32,
}

#[derive(Deserialize)]
pub struct UIField {
    body_info_font_size: u16,
    show_fps:            bool,

    show_energy:             bool,
    show_division_threshold: bool,
    show_body_type:          bool,
    show_lifespan:           bool,
    show_skills:             bool,
    show_viruses:            bool,
}

#[derive(Deserialize)]
pub struct ConditionsField {
    fewer_plants_chance: f32,
    more_plants_chance:  f32,
}

#[derive(Deserialize)]
struct Data {
    body:       BodyField,
    plants:     PlantField,
    energy:     EnergyField,
    viruses:    VirusesField,
    conditions: ConditionsField,
    ui:         UIField,
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
    let conditions = config.conditions;
    let ui = config.ui;

    unsafe {
        // Body-related
        BODIES_N = body.bodies_n;
        PASSIVE_CHANCE = body.passive_chance;
        AVERAGE_ENERGY = body.average_energy;
        AVERAGE_SPEED = body.average_speed;
        AVERAGE_DIVISION_THRESHOLD = body.average_division_threshold;
        AVERAGE_VISION_DISTANCE = body.average_vision_distance;
        CONST_FOR_LIFESPAN = body.const_for_lifespan;

        SKILLS_CHANGE_CHANCE = body.skills_change_chance;
        DEVIATION = body.deviation;
        LIFESPAN = body.lifespan;
        MIN_ENERGY = body.min_energy;
        CROSS_LIFESPAN = body.cross_lifespan;

        // Plants-related
        PLANTS_DENSITY = plants.plants_density;
        PLANT_SPAWN_CHANCE = plants.plant_spawn_chance;
        PLANT_DIE_CHANCE = plants.plant_die_chance;

        // Virus-related
        SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE =
            viruses.speedvirus_first_generation_infection_chance;
        SPEEDVIRUS_SPEED_DECREASE = viruses.speedvirus_speed_decrease;
        SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING =
            viruses.speedvirus_energy_spent_for_healing;
        SPEEDVIRUS_HEAL_ENERGY = viruses.speedvirus_heal_energy;

        VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE =
            viruses.visionvirus_first_generation_infection_chance;
        VISIONVIRUS_VISION_DISTANCE_DECREASE =
            viruses.visionvirus_vision_distance_decrease;
        VISIONVIRUS_ENERGY_SPENT_FOR_HEALING =
            viruses.visionvirus_energy_spent_for_healing;
        VISIONVIRUS_HEAL_ENERGY = viruses.visionvirus_heal_energy;

        // Energy-related
        ENERGY_SPENT_CONST_FOR_MASS =
            energy.energy_spent_const_for_mass;
        ENERGY_SPENT_CONST_FOR_SKILLS =
            energy.energy_spent_const_for_skills;
        ENERGY_SPENT_CONST_FOR_VISION_DISTANCE =
            energy.energy_spent_const_for_vision_distance;
        ENERGY_SPENT_CONST_FOR_MOVEMENT =
            energy.energy_spent_const_for_movement;

        // Conditions
        FEWER_PLANTS_CHANCE = conditions.fewer_plants_chance;
        MORE_PLANTS_CHANCE = conditions.more_plants_chance;

        // UI-related
        BODY_INFO_FONT_SIZE = ui.body_info_font_size;
        SHOW_FPS = ui.show_fps;

        SHOW_ENERGY = ui.show_energy;
        SHOW_DIVISION_THRESHOLD = ui.show_division_threshold;
        SHOW_BODY_TYPE = ui.show_body_type;
        SHOW_LIFESPAN = ui.show_lifespan;
        SHOW_SKILLS = ui.show_skills;
        SHOW_VIRUSES = ui.show_viruses;
    };
}
