use crate::constants::*;
use macroquad::prelude::*;
use serde_derive::Deserialize;
use std::{fs::read_to_string, ops::Range, process::exit};
use toml::from_str;

// Average spawn attributes
pub static mut OMNIVOROUS_N: usize = 0;
pub static mut HERBIVOROUS_N: usize = 0;
pub static mut CARNIVOROUS_N: usize = 0;

pub static mut AVERAGE_ENERGY_OMNIVOROUS_HERBIVOROUS: f32 = 0.0;
pub static mut AVERAGE_ENERGY_CARNIVOROUS: f32 = 0.0;

pub static mut AVERAGE_SPEED: f32 = 0.0;

pub static mut AVERAGE_DIVISION_THRESHOLD_OMNIVOROUS_HERBIVOROUS:
    f32 = 0.0;
pub static mut AVERAGE_DIVISION_THRESHOLD_CARNIVOROUS: f32 = 0.0;

pub static mut AVERAGE_VISION_DISTANCE: f32 = 0.0;

pub static mut OMNIVOROUS_FOOD_PART: f32 = 0.0;
pub static mut CARNIVOROUS_ENERGY_CONST: f32 = 0.0;

pub static mut SKILLS_CHANGE_CHANCE: f32 = 0.0;
pub static mut PLANTS_DENSITY: f32 = 0.0;

pub static mut DEVIATION: f32 = 0.0;
pub static mut LIFESPAN: f32 = 0.0;
pub static mut MIN_ENERGY: f32 = 0.0;

pub static mut PLANT_SPAWN_CHANCE: f32 = 0.0;
pub static mut PLANT_DIE_CHANCE: f32 = 0.0;
pub static mut CONST_FOR_LIFESPAN: f32 = 0.0;

// Death
pub static mut CROSS_LIFESPAN: u64 = 0;

// Spending energy
pub static mut ENERGY_SPENT_CONST_FOR_MASS: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_SKILLS: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_VISION_DISTANCE: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_MOVEMENT: f32 = 0.0;

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

// Condition
pub static mut CONDITION_CHANCE: f32 = 0.0;
pub static mut CONDITION_LIFETIME: Range<u64> = 0..0;

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
    omnivorous_n:                                      usize,
    herbivorous_n:                                     usize,
    carnivorous_n:                                     usize,
    average_energy_omnivorous_herbivorous:             f32,
    average_energy_carnivorous:                        f32,
    average_speed:                                     f32,
    average_division_threshold_omnivorous_herbivorous: f32,
    average_division_threshold_carnivorous:            f32,
    average_vision_distance:                           f32,
    omnivorous_food_part:                              f32,
    carnivorous_energy_const:                          f32,
    skills_change_chance:                              f32,
    deviation:                                         f32,
    lifespan:                                          f32,
    min_energy:                                        f32,
    cross_lifespan:                                    u64,
    const_for_lifespan:                                f32,
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
    speedvirus_first_generation_infection_chance:  f32,
    speedvirus_speed_decrease:                     f32,
    speedvirus_energy_spent_for_healing:           f32,
    speedvirus_heal_energy:                        f32,
    visionvirus_first_generation_infection_chance: f32,
    visionvirus_vision_distance_decrease:          f32,
    visionvirus_energy_spent_for_healing:          f32,
    visionvirus_heal_energy:                       f32,
}

#[derive(Deserialize)]
pub struct UIField {
    body_info_font_size:     u16,
    show_fps:                bool,
    show_energy:             bool,
    show_division_threshold: bool,
    show_body_type:          bool,
    show_lifespan:           bool,
    show_skills:             bool,
    show_viruses:            bool,
}

#[derive(Deserialize)]
pub struct ConditionField {
    condition_chance:   f32,
    condition_lifetime: [u64; 2],
}

#[derive(Deserialize)]
struct Data {
    body:      BodyField,
    plants:    PlantField,
    energy:    EnergyField,
    viruses:   VirusesField,
    condition: ConditionField,
    ui:        UIField,
}

pub fn config_setup(first_run: bool) {
    let contents = match read_to_string(CONFIG_FILE_NAME) {
        Ok(contents) => contents,
        Err(_) => {
            if first_run {
                eprintln!("The config file hasn't been found.");
                exit(1);
            } else {
                return;
            }
        }
    };

    let config: Data = match from_str(&contents) {
        Ok(config) => config,
        Err(_) => {
            if first_run {
                eprintln!("The file style isn't correct.");
                exit(1);
            } else {
                return;
            }
        }
    };

    let body = config.body;
    let plants = config.plants;
    let energy = config.energy;
    let viruses = config.viruses;
    let condition = config.condition;
    let ui = config.ui;

    unsafe {
        // Body-related
        OMNIVOROUS_N = body.omnivorous_n;
        HERBIVOROUS_N = body.herbivorous_n;
        CARNIVOROUS_N = body.carnivorous_n;

        AVERAGE_ENERGY_OMNIVOROUS_HERBIVOROUS =
            body.average_energy_omnivorous_herbivorous;
        AVERAGE_ENERGY_CARNIVOROUS = body.average_energy_carnivorous;

        AVERAGE_SPEED = body.average_speed;

        AVERAGE_DIVISION_THRESHOLD_OMNIVOROUS_HERBIVOROUS =
            body.average_division_threshold_omnivorous_herbivorous;
        AVERAGE_DIVISION_THRESHOLD_CARNIVOROUS =
            body.average_division_threshold_carnivorous;

        AVERAGE_VISION_DISTANCE = body.average_vision_distance;

        OMNIVOROUS_FOOD_PART = body.omnivorous_food_part;
        CARNIVOROUS_ENERGY_CONST = body.carnivorous_energy_const;

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

        // Condition
        CONDITION_CHANCE = condition.condition_chance;
        CONDITION_LIFETIME = condition.condition_lifetime[0]
            ..condition.condition_lifetime[1];

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
