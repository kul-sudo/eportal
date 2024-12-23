use crate::constants::*;
use macroquad::prelude::*;
use serde_derive::Deserialize;
use std::{
    fs::read_to_string,
    ops::Range,
    process::exit,
    sync::{LazyLock, RwLock},
};
use toml::from_str;

#[derive(Default, Debug)]
// Average spawn attributes
pub struct UserConstants {
    pub omnivorous_n:                                      usize,
    pub herbivorous_n:                                     usize,
    pub carnivorous_n:                                     usize,
    pub average_energy_omnivorous_herbivorous:             f32,
    pub average_energy_carnivorous:                        f32,
    pub average_speed:                                     f32,
    pub average_division_threshold_omnivorous_herbivorous: f32,
    pub average_division_threshold_carnivorous:            f32,
    pub average_vision_distance:                           f32,
    pub omnivorous_food_part:                              f32,
    pub carnivorous_energy_const:                          f32,
    pub skills_change_chance:                              f32,
    pub plants_density:                                    f32,
    pub deviation:                                         f32,
    pub lifespan:                                          f32,
    pub min_energy:                                        f32,
    pub plant_spawn_chance:                                f32,
    pub plant_die_chance:                                  f32,
    pub const_for_lifespan:                                f32,
    pub cross_lifespan:                                    u64,
    pub energy_spent_const_for_mass:                       f32,
    pub energy_spent_const_for_skills:                     f32,
    pub energy_spent_const_for_vision_distance:            f32,
    pub energy_spent_const_for_movement:                   f32,
    pub speedvirus_first_generation_infection_chance:      f32,
    pub speedvirus_speed_decrease:                         f32,
    pub speedvirus_energy_spent_for_healing:               f32,
    pub speedvirus_heal_energy:                            f32,
    pub visionvirus_first_generation_infection_chance:     f32,
    pub visionvirus_vision_distance_decrease:              f32,
    pub visionvirus_energy_spent_for_healing:              f32,
    pub visionvirus_heal_energy:                           f32,
    pub condition_chance:                                  f32,
    pub condition_lifetime:                                Range<u64>,
    pub body_info_font_size:                               u16,
    pub show_fps:                                          bool,
    pub show_energy:                                       bool,
    pub show_division_threshold:                           bool,
    pub show_body_type:                                    bool,
    pub show_lifespan:                                     bool,
    pub show_skills:                                       bool,
    pub show_viruses:                                      bool,
}

pub static USER_CONSTANTS: LazyLock<RwLock<UserConstants>> =
    LazyLock::new(|| RwLock::new(Default::default()));

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

    let mut user_constants = USER_CONSTANTS.write().unwrap();
    *user_constants = UserConstants {
        omnivorous_n:                                      body
            .omnivorous_n,
        herbivorous_n:                                     body
            .herbivorous_n,
        carnivorous_n:                                     body
            .carnivorous_n,
        average_energy_omnivorous_herbivorous:             body
            .average_energy_omnivorous_herbivorous,
        average_energy_carnivorous:                        body
            .average_energy_carnivorous,
        average_speed:                                     body
            .average_speed,
        average_division_threshold_omnivorous_herbivorous: body
            .average_division_threshold_omnivorous_herbivorous,
        average_division_threshold_carnivorous:            body
            .average_division_threshold_carnivorous,
        average_vision_distance:                           body
            .average_vision_distance,
        omnivorous_food_part:                              body
            .omnivorous_food_part,

        carnivorous_energy_const:                      body
            .carnivorous_energy_const,
        const_for_lifespan:                            body
            .const_for_lifespan,
        skills_change_chance:                          body
            .skills_change_chance,
        deviation:                                     body.deviation,
        lifespan:                                      body.lifespan,
        min_energy:                                    body
            .min_energy,
        cross_lifespan:                                body
            .cross_lifespan,
        plants_density:                                plants
            .plants_density,
        plant_spawn_chance:                            plants
            .plant_spawn_chance,
        plant_die_chance:                              plants
            .plant_die_chance,
        speedvirus_first_generation_infection_chance:  viruses
            .speedvirus_first_generation_infection_chance,
        speedvirus_speed_decrease:                     viruses
            .speedvirus_speed_decrease,
        speedvirus_energy_spent_for_healing:           viruses
            .speedvirus_energy_spent_for_healing,
        speedvirus_heal_energy:                        viruses
            .speedvirus_heal_energy,
        visionvirus_first_generation_infection_chance: viruses
            .visionvirus_first_generation_infection_chance,
        visionvirus_vision_distance_decrease:          viruses
            .visionvirus_vision_distance_decrease,
        visionvirus_energy_spent_for_healing:          viruses
            .visionvirus_energy_spent_for_healing,
        visionvirus_heal_energy:                       viruses
            .visionvirus_heal_energy,
        energy_spent_const_for_mass:                   energy
            .energy_spent_const_for_mass,
        energy_spent_const_for_skills:                 energy
            .energy_spent_const_for_skills,
        energy_spent_const_for_vision_distance:        energy
            .energy_spent_const_for_vision_distance,
        energy_spent_const_for_movement:               energy
            .energy_spent_const_for_movement,
        condition_chance:                              condition
            .condition_chance,
        condition_lifetime:                            condition
            .condition_lifetime[0]
            ..condition.condition_lifetime[1],
        body_info_font_size:                           ui
            .body_info_font_size,
        show_fps:                                      ui.show_fps,
        show_energy:                                   ui.show_energy,
        show_division_threshold:                       ui
            .show_division_threshold,
        show_body_type:                                ui
            .show_body_type,
        show_lifespan:                                 ui
            .show_lifespan,
        show_skills:                                   ui.show_skills,
        show_viruses:                                  ui
            .show_viruses,
    };
}
