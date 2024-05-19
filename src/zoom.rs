use macroquad::{
    camera::{set_camera, Camera, Camera2D},
    math::{vec2, Rect, Vec2},
    window::{screen_height, screen_width},
};

use crate::{MAX_ZOOM, MIN_ZOOM};

/// Adjust the coordinates according to the borders.
macro_rules! adjusted_coordinates {
    ($pos:expr, $area_size:expr) => {
        Vec2 {
            x: ($pos.x * MAX_ZOOM)
                .max($area_size.x / MAX_ZOOM / 2.0)
                .min($area_size.x * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
            y: ($pos.y * MAX_ZOOM)
                .max($area_size.y / MAX_ZOOM / 2.0)
                .min($area_size.y * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        }
    };
}

#[derive(Clone, Copy)]
pub struct Zoom {
    pub width: f32,
    pub height: f32,
    pub diagonal: f32,
    pub center_pos: Option<Vec2>,
    pub mouse_pos: Option<Vec2>,
}

/// Set the camera zoom to where the mouse cursor is.
pub fn get_zoom_target(camera: &mut Camera2D, area_size: Vec2, zoom: &mut Zoom) {
    zoom.center_pos = Some(adjusted_coordinates!(zoom.mouse_pos.unwrap(), area_size));

    // camera.viewport = Some((300, 300, 1920, 1080));
    camera.target = zoom.center_pos.unwrap();
    camera.zoom = vec2(zoom.width, zoom.height);
    set_camera(camera);
}

/// Reset the camera zoom.
pub fn default_camera(camera: &mut Camera2D, area_size: Vec2) {
    camera.target = vec2(area_size.x / 2.0, area_size.y / 2.0);
    camera.zoom = vec2(MIN_ZOOM / area_size.x * 2.0, MIN_ZOOM / area_size.y * 2.0);
    set_camera(camera);
}
