use macroquad::{
    camera::{set_camera, Camera2D},
    math::{vec2, Rect, Vec2},
};

use crate::{adjusted_pos, constants::*};

#[derive(Clone, Copy)]
pub struct Zoom {
    /// Th width of the part of the area size the zoom shows.
    pub scaling_width: f32,
    /// The height of the part of the area size the zoom shows.
    pub scaling_height: f32,
    pub width: f32,
    pub height: f32,
    pub center_pos: Option<Vec2>,
    pub mouse_pos: Option<Vec2>,
    /// The rectangle on the same place as the camera.
    pub rect: Option<Rect>,
    /// Normal rect size + OBJECT_RADIUS * 2.0.
    pub extended_rect: Option<Rect>,
    /// The diagonal of the normal rectangle.
    pub diagonal_rect: f32,
    /// The diagonal of the extended rectangle.
    pub diagonal_extended_rect: f32,
}

/// Set the camera zoom to where the mouse cursor is.
pub fn get_zoom_target(camera: &mut Camera2D, area_size: &Vec2, zoom: &mut Zoom) {
    zoom.center_pos = Some(adjusted_pos!(zoom.mouse_pos.unwrap(), area_size));
    zoom.rect = Some(Rect::new(
        zoom.center_pos.unwrap().x - zoom.width / 2.0,
        zoom.center_pos.unwrap().y - zoom.height / 2.0,
        zoom.width,
        zoom.height,
    ));

    zoom.extended_rect = Some(Rect::new(
        zoom.center_pos.unwrap().x - zoom.width / 2.0 - OBJECT_RADIUS,
        zoom.center_pos.unwrap().y - zoom.height / 2.0 - OBJECT_RADIUS,
        zoom.width + OBJECT_RADIUS * 2.0,
        zoom.height + OBJECT_RADIUS * 2.0,
    ));

    camera.target = zoom.center_pos.unwrap();
    camera.zoom = vec2(zoom.scaling_width, zoom.scaling_height);
    set_camera(camera);
}

/// Reset the camera zoom.
pub fn default_camera(camera: &mut Camera2D, area_size: &Vec2) {
    camera.target = vec2(area_size.x / 2.0, area_size.y / 2.0);
    camera.zoom = vec2(MIN_ZOOM / area_size.x * 2.0, MIN_ZOOM / area_size.y * 2.0);
    set_camera(camera);
}
