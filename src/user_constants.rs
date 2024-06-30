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
pub static mut LIFESPAN: f32 = 0.0; // In seconds
pub static mut MIN_ENERGY: f32 = 0.0;

// Death
pub static mut CROSS_LIFESPAN: u64 = 0; // In seconds

// Spending energy
pub static mut ENERGY_SPENT_CONST_FOR_MASS: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_SKILLS: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_VISION_DISTANCE: f32 = 0.0;
pub static mut ENERGY_SPENT_CONST_FOR_MOVEMENT: f32 = 0.0;
pub static mut CONST_FOR_LIFESPAN: f32 = 0.0;

// SpeedVirus
pub static mut SPEEDVIRUS_FIRST_GENERATION_INFECTION_CHANCE: f32 = 0.0;
pub static mut SPEEDVIRUS_SPEED_DECREASE: f32 = 0.0;
pub static mut SPEEDVIRUS_ENERGY_SPENT_FOR_HEALING: f32 = 0.0;
pub static mut SPEEDVIRUS_HEAL_ENERGY: f32 = 0.0;

// VisionVirus
pub static mut VISIONVIRUS_FIRST_GENERATION_INFECTION_CHANCE: f32 = 0.0;
pub static mut VISIONVIRUS_VISION_DISTANCE_DECREASE: f32 = 0.0;
pub static mut VISIONVIRUS_ENERGY_SPENT_FOR_HEALING: f32 = 0.0;
pub static mut VISIONVIRUS_HEAL_ENERGY: f32 = 0.0;

// UI
pub static mut BODY_INFO_FONT_SIZE: u16 = 0;

pub static mut SHOW_ENERGY: bool = false;
pub static mut SHOW_DIVISION_THRESHOLD: bool = false;
pub static mut SHOW_BODY_TYPE: bool = false;
pub static mut SHOW_LIFESPAN: bool = false;
pub static mut SHOW_SKILLS: bool = false;
pub static mut SHOW_VIRUSES: bool = false;
