#![feature(core_intrinsics)]
#![feature(more_float_constants)]

mod body;
mod constants;
mod plant;

use body::*;
use constants::*;
use plant::{randomly_spawn_plant, Plant};

use std::{
    collections::{HashMap, HashSet},
    env::consts::OS,
    f32::consts::SQRT_2,
    intrinsics::unlikely,
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::{set_camera, Camera2D},
    color::{GREEN, RED, WHITE},
    input::{is_mouse_button_pressed, mouse_position},
    math::{vec2, Rect, Vec2},
    miniquad::{window::set_fullscreen, MouseButton},
    shapes::{draw_circle, draw_line, draw_rectangle, draw_triangle},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

macro_rules! adjusted_coordinates {
    ($pos:expr, $area_size:expr) => {
        (
            ($pos.x * MAX_ZOOM)
                .max($area_size.x / MAX_ZOOM / 2.0)
                .min($area_size.x * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
            ($pos.y * MAX_ZOOM)
                .max($area_size.y / MAX_ZOOM / 2.0)
                .min($area_size.y * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        )
    };
}

fn get_zoom_target(camera: &mut Camera2D, area_size: Vec2) {
    let (x, y) = mouse_position();
    let (target_x, target_y) = adjusted_coordinates!(Vec2 { x, y }, area_size);

    camera.target = vec2(target_x, target_y);
    camera.zoom = vec2(MAX_ZOOM / area_size.x * 2.0, MAX_ZOOM / area_size.y * 2.0);
    set_camera(camera);
}

fn default_camera(camera: &mut Camera2D, area_size: Vec2) {
    camera.target = vec2(area_size.x / 2.0, area_size.y / 2.0);
    camera.zoom = vec2(MIN_ZOOM / area_size.x * 2.0, MIN_ZOOM / area_size.y * 2.0);
    set_camera(camera);
}

fn get_nearest_plant_for_body(plants: &[Plant], body: &Body) -> Option<(f32, (usize, Plant))> {
    let (plant_id, plant) = plants
        .iter()
        .enumerate()
        .min_by_key(|(_, plant)| plant.pos.distance(body.pos) as i16)?;
    Some((plant.pos.distance(body.pos), (plant_id, *plant)))
}

fn get_with_deviation(value: f32, rng: &mut ThreadRng) -> f32 {
    let part = value * DEVIATION;
    rng.gen_range(value - part..value + part)
}

// fn get_nearest_body_for_body<'a>(
//     bodies: &'a HashMap<usize, Body<'a>>,
//     body: &Body,
// ) -> Option<(f32, (usize, &'a Body<'a>))> {
//     let (body_id, closest_body) = bodies.iter().min_by_key(|(_, enemy_body)| {
//         distance(vec![enemy_body.x, enemy_body.y], vec![body.x, body.y]) as isize
//     })?;
//     Some((
//         distance(vec![closest_body.x, closest_body.y], vec![body.x, body.y]),
//         (*body_id, closest_body),
//     ))
// }

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Make the window fullscreen on Linux
    if OS == "linux" {
        set_fullscreen(true);
        sleep(Duration::from_secs(1));
        next_frame().await;
    }

    let area_size = Vec2 {
        x: screen_width() * OBJECT_RADIUS,
        y: screen_height() * OBJECT_RADIUS,
    };

    let mut bodies: HashMap<usize, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: Vec<Plant> = Vec::with_capacity(PLANTS_N);

    let rng = &mut thread_rng();

    for i in 1..BODIES_N {
        randomly_spawn_body(
            &mut bodies,
            area_size,
            if i >= BODY_EATERS_N {
                EatingStrategy::Plants
            } else {
                EatingStrategy::Bodies
            },
        );
    }

    // Spawn the plants
    for _ in 0..PLANTS_N {
        randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
    }

    // // Get the nearest plant for each spawned body
    // for body in bodies.values_mut() {
    //     body.nearest_plant = get_nearest_plant_for_body(&plants, body);
    //     // body.nearest_body = get_nearest_body_for_body(&bodies, body)
    // }

    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.x, area_size.y));
    default_camera(&mut camera, area_size);

    let mut zoom_mode = false;
    let mut last_updated = Instant::now();

    loop {
        if unlikely(is_mouse_button_pressed(MouseButton::Left)) {
            if zoom_mode {
                default_camera(&mut camera, area_size);
            } else {
                get_zoom_target(&mut camera, area_size);
            }

            zoom_mode = !zoom_mode
        }

        if zoom_mode {
            get_zoom_target(&mut camera, area_size);
        }

        if unlikely(rng.gen_range(0.0..1.0) > 1.0 - PLANT_SPAWN_CHANCE) {
            randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
        }

        let mut bodies_to_iter = bodies.iter_mut().collect::<Vec<_>>();
        bodies_to_iter.shuffle(rng);

        let is_draw_mode =
            last_updated.elapsed().as_millis() >= Duration::from_secs(1 / FPS).as_millis();

        let mut bodies_to_delete: HashSet<usize> = HashSet::with_capacity(bodies_to_iter.len());

        for (body_id, body) in bodies_to_iter {
            if body.energy <= ALIVE_MIN_ENERGY {
                match body.death_time {
                    Some(timestamp) => {
                        if timestamp.elapsed().as_secs() >= CROSS_LIFESPAN {
                            bodies_to_delete.insert(*body_id);
                        }
                    }
                    None => {
                        body.death_time = Some(Instant::now());
                    }
                }
            } else {
                body.energy -=
                    ENERGY_SPEND_CONST_FOR_MASS * body.energy + ENERGY_SPEND_CONST_FOR_IQ * body.iq;
            }
            // body.nearest_plant = get_nearest_plant_for_body(&plants, body);
            // body.nearest_body = get_nearest_body_for_body(&mut bodies, body);
            // update_nearest_body(body, &bodies);

            // body.energy -= ENERGY_FOR_WALKING;
            // match body.preference {
            //     Preference::Plants => {
            //         // Move towards the nearest plant
            //         let (distance_to_plant, (plant_id, nearest_plant)) =
            //             body.nearest_plant.unwrap();
            //         let (dx, dy) = (nearest_plant.x - body.x, nearest_plant.y - body.y);
            //         let coeff = body.speed / distance_to_plant;
            //
            //         body.x += coeff * dx;
            //         body.y += coeff * dy;
            //         body.status = Status::FollowingPlant(nearest_plant);
            //
            //         if zoom_mode && is_draw_mode {
            //             draw_line(body.x, body.y, nearest_plant.x, nearest_plant.y, 5.0, WHITE);
            //         }
            //
            //         // If there's been a contact between the body and a plant, handle it
            //         if distance!([body.x, body.y], [nearest_plant.x, nearest_plant.y])
            //             <= OBJECT_RADIUS
            //         {
            //             body.energy += PLANT_HP;
            //             plants.remove(plant_id);
            //             body.nearest_plant = None;
            //             body.status = Status::Sleeping
            //         }
            //     }
            //     Preference::Bodies => {
            //         //     for (enemy_id, mut enemy_body) in bodies.clone().into_iter() {
            //         //         if (enemy_body.x, enemy_body.y) == (body.x, body.y) {
            //         //             bodies.remove(match body.hp.cmp(&enemy_body.hp) {
            //         //                 Ordering::Less => {
            //         //                     enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //         //                     body_id
            //         //                 }
            //         //                 Ordering::Equal => {
            //         //                     let options = [body_id, &enemy_id];
            //         //                     let chosen = options.choose(rng).unwrap();
            //         //
            //         //                     if *chosen == body_id {
            //         //                         enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //         //                     } else {
            //         //                         body.hp += enemy_body.hp.min(MAX_HP - body.hp);
            //         //                     }
            //         //
            //         //                     chosen
            //         //                 }
            //         //                 Ordering::Greater => {
            //         //                     enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //         //                     &enemy_id
            //         //                 }
            //         //             });
            //         //         }
            //         //     }
            //     }
            // }
            //
            // if is_draw_mode {
            //     draw_text(
            //         &body.energy.to_string(),
            //         body.pos.x
            //             - measure_text(&body_id.to_string(), None, BODY_INFO_FONT_SIZE, 1.0).width
            //                 / 2.0,
            //         body.pos.y - OBJECT_RADIUS - MIN_GAP,
            //         BODY_INFO_FONT_SIZE as f32,
            //         WHITE,
            //     );

            //     draw_body(body);
            //     // draw_text(&format!("zoom {}", zoom), 10.0, 20.0, 20.0, WHITE);
            // }
        }

        for body in &bodies_to_delete {
            bodies.remove(body);
        }
        bodies_to_delete.clear();

        if is_draw_mode {
            // draw_rectangle_lines(0.0, 0.0, area_size.x, area_size.y, 30.0, WHITE);
            for plant in &plants {
                draw_plant!(plant);
            }

            for (body_id, body) in &bodies {
                let to_display = body_id;
                draw_text(
                    &to_display.to_string(),
                    body.pos.x
                        - measure_text(&to_display.to_string(), None, BODY_INFO_FONT_SIZE, 1.0)
                            .width
                            / 2.0,
                    body.pos.y - OBJECT_RADIUS - MIN_GAP,
                    BODY_INFO_FONT_SIZE as f32,
                    WHITE,
                );

                draw_body!(body);
            }
            // draw_text(&format!("zoom {}", zoom), 10.0, 20.0, 20.0, WHITE);

            // if zoom_mode {
            //     let mouse_position = mouse_position();
            //     let (x, y) = adjusted_coordinates!(
            //         mouse_position.0 + 25.0,
            //         mouse_position.1 - 25.0,
            //         area_size
            //     );
            //     draw_text("zoomed in", x, y, 10.0 * MAX_ZOOM, WHITE)
            // }
        }

        if is_draw_mode {
            last_updated = Instant::now();
            next_frame().await;
        }
    }
}
