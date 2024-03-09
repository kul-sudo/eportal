// Base
pub static BODIES_N: usize = 200;
pub static PLANTS_N: usize = 2000;
pub static PLANT_HP: f32 = 15.0;
pub static ENERGY_FOR_WALKING: f32 = 0.1;
pub static OBJECT_RADIUS: f32 = 10.0;
pub static MIN_GAP: f32 = 3.0;
pub static COLOR_GAP: f32 = 0.7; // Depends on the color limit
pub static PLANT_SPAWN_CHANCE: f32 = 1.0; // Mid-game
pub static PLANT_SPAWN_TIME_LIMIT: u64 = 1; // In millis

pub static FPS: u64 = 144;

// Zoom
pub static MAX_ZOOM: f32 = OBJECT_RADIUS;
pub static MIN_ZOOM: f32 = 1.0;
pub static MOUSE_WHEEL_ZOOM_DIVISION: f32 = 10.0;
pub static BODY_JUMP_DELAY: u64 = 150; // In millis

// UI
pub static BODY_INFO_FONT_SIZE: u16 = 17;

// Math
pub static ROOT_OF_3_DIVIDED_BY_2: f32 = 0.8660254;
