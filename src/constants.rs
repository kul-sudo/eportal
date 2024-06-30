use std::f32::consts::SQRT_3;

pub const DEFAULT_SCREEN_WIDTH: f32 = 1920.0;
pub const DEFAULT_SCREEN_HEIGHT: f32 = 1080.0;

// TOML
pub const CONFIG_FILE_NAME: &str = "config.toml";

// Base
pub const DEFAULT_CELL_ROWS: usize = 129; // May be needed to be changed when the perfect values for the
                                          // evolution process have been determined
pub const DEFAULT_PLANTS_N: usize = 52897;
pub static mut PLANTS_N: usize = 0;
pub const MAX_PLANTS_IN_ONE_CELL: usize = 10;

/// The recommended constants in config.toml have been detemined for this area space.
pub static DEFAULT_AREA_SIZE_RATIO: f32 = DEFAULT_SCREEN_WIDTH / DEFAULT_SCREEN_HEIGHT;

pub const OBJECT_RADIUS: f32 = 10.0;
/// Used for `get_with_deviation`.
pub const COLOR_MIN: u8 = 50;
pub const COLOR_MAX: u8 = 250;

// Evolution process
pub const PLANT_ENERGY: f32 = 100.0;
pub const MIN_GAP: f32 = 3.0;
pub const COLOR_GAP: f32 = 0.63; // Depends on COLOR_MIN and COLOR_MAX
pub const PLANT_SPAWN_TIME_LIMIT: u64 = 5; // In millis
pub const MIN_TO_REMOVE: usize = 500; // Bodies and plants are removed only it's needed to remove
                                      // more of them than this constant. That's because when the amount of object to remove, the time it
                                      // takes to delete them barely depends on their amount

// Plants
pub static mut PLANTS_N_FOR_ONE_STEP: usize = 0;

// Zoom
pub const MAX_ZOOM: f32 = OBJECT_RADIUS;
pub const MIN_ZOOM: f32 = 1.0;

// Math
pub static COSINE_OF_30_DEGREES: f32 = SQRT_3 / 2.0;

// Misc
pub const FPS: u64 = 144;
