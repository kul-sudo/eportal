#![feature(core_intrinsics)]

mod constants;

use constants::*;

use std::{
    collections::HashMap,
    intrinsics::unlikely,
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::{set_camera, Camera, Camera2D},
    color::{Color, GREEN, WHITE},
    input::{is_mouse_button_pressed, mouse_position},
    math::{vec2, Rect, Vec2, Vec3},
    miniquad::{window::set_fullscreen, MouseButton},
    rand::gen_range,
    shapes::{draw_circle, draw_line, draw_rectangle_lines, draw_triangle},
    text::{draw_text, draw_text_ex, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status<'a> {
    FollowingBody(&'a Body<'a>),
    FollowingPlant(&'a Plant),
    EscapingBody(&'a Body<'a>),
    Sleeping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IQStage {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EatingStrategy {
    Bodies,
    Plants,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Body<'a> {
    pos: Vec2,
    energy: f32,
    speed: f32,
    vision_distance: f32,
    eating_strategy: EatingStrategy,
    division_threshold: f32,
    mass: f32,
    iq: IQStage,
    color: Color,
    status: Status<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plant {
    pos: Vec2,
}

macro_rules! adjusted_coordinates {
    ($pos:expr, $area_size:expr) => {{
        (
            ($pos.x * MAX_ZOOM)
                .max($area_size.0 / MAX_ZOOM / 2.0)
                .min($area_size.0 * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
            ($pos.y * MAX_ZOOM)
                .max($area_size.1 / MAX_ZOOM / 2.0)
                .min($area_size.1 * (1.0 - 1.0 / (2.0 * MAX_ZOOM))),
        )
    }};
}

macro_rules! draw_body {
    ($pos:expr, $color:expr) => {
        draw_circle($pos.x, $pos.y, OBJECT_RADIUS, $color);
    };
}

macro_rules! draw_plant {
    ($plant:expr) => {
        draw_triangle(
            Vec2 {
                x: $plant.pos.x,
                y: $plant.pos.y - OBJECT_RADIUS,
            },
            Vec2 {
                x: $plant.pos.x + OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: $plant.pos.y + OBJECT_RADIUS / 2.0,
            },
            Vec2 {
                x: $plant.pos.x - OBJECT_RADIUS * (COSINE_OF_30_DEGREES),
                y: $plant.pos.y + OBJECT_RADIUS / 2.0,
            },
            GREEN,
        );
    };
}

fn get_zoom_target(camera: &mut Camera2D, area_size: (f32, f32)) {
    let mouse_position = mouse_position();
    let (target_x, target_y) = adjusted_coordinates!(
        Vec2 {
            x: mouse_position.0,
            y: mouse_position.1
        },
        area_size
    );

    camera.target = vec2(target_x, target_y);
    camera.zoom = vec2(MAX_ZOOM / area_size.0 * 2.0, MAX_ZOOM / area_size.1 * 2.0);
    set_camera(camera);
}

fn default_camera(camera: &mut Camera2D, area_size: (f32, f32)) {
    camera.target = vec2(area_size.0 / 2.0, area_size.1 / 2.0);
    camera.zoom = vec2(MIN_ZOOM / area_size.0 * 2.0, MIN_ZOOM / area_size.1 * 2.0);
    set_camera(camera);
}

fn get_nearest_plant_for_body(plants: &[Plant], body: &Body) -> Option<(f32, (usize, Plant))> {
    let (plant_id, plant) = plants
        .iter()
        .enumerate()
        .min_by_key(|(_, plant)| plant.pos.distance(body.pos) as isize)?;
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
fn randomly_spawn_plant(
    bodies: &mut HashMap<usize, Body>,
    plants: &mut Vec<Plant>,
    rng: &mut ThreadRng,
    area_size: (f32, f32),
) {
    let starting_point = Instant::now();

    let mut x;
    let mut y;

    while {
        if starting_point.elapsed().as_nanos()
            >= Duration::from_millis(PLANT_SPAWN_TIME_LIMIT).as_nanos()
        {
            return;
        }
        x = rng.gen_range(0.0..area_size.0);
        y = rng.gen_range(0.0..area_size.1);
        (x <= OBJECT_RADIUS + MIN_GAP || x >= area_size.0 - OBJECT_RADIUS - MIN_GAP)
            || (y <= OBJECT_RADIUS + MIN_GAP || y >= area_size.1 - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| body.pos.distance(Vec2 { x, y }) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
            || plants
                .iter()
                .any(|plant| plant.pos.distance(Vec2 { x, y }) <= OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    plants.push(Plant { pos: Vec2 { x, y } });
}

fn spawn_body(
    bodies: &mut HashMap<usize, Body>,
    pos: Vec2,
    energy: f32,
    speed: f32,
    vision_distance: f32,
    eating_strategy: EatingStrategy,
    division_threshold: f32,
    mass: f32,
    iq: IQStage,
    color: Color,
) {
    bodies.insert(
        bodies.len() + 1,
        Body {
            pos,
            energy,
            speed,
            vision_distance,
            eating_strategy,
            division_threshold,
            mass,
            iq,
            color,
            status: Status::Sleeping,
        },
    );
}

fn randomly_spawn_body(
    bodies: &mut HashMap<usize, Body>,
    area_size: (f32, f32),
    eating_strategy: EatingStrategy,
) {
    let rng = &mut thread_rng();

    let mut x;
    let mut y;

    while {
        x = rng.gen_range(0.0..area_size.0);
        y = rng.gen_range(0.0..area_size.1);
        (x <= OBJECT_RADIUS + MIN_GAP || x >= area_size.0 - OBJECT_RADIUS - MIN_GAP)
            || (y <= OBJECT_RADIUS + MIN_GAP || y >= area_size.1 - OBJECT_RADIUS - MIN_GAP)
            || bodies
                .values()
                .any(|body| body.pos.distance(Vec2 { x, y }) < OBJECT_RADIUS * 2.0 + MIN_GAP)
    } {}

    let real_color_gap = COLOR_GAP / ((BODIES_N + 1) as f32).powf(1.0 / 3.0);

    let mut color = Color::from_rgba(
        gen_range(50, 250),
        gen_range(50, 250),
        gen_range(50, 250),
        255,
    );

    while bodies.values().any(|body| {
        let current_body_rgb = Vec3 {
            x: body.color.r,
            y: body.color.g,
            z: body.color.b,
        };
        let green_rgb = Vec3 {
            x: GREEN.r,
            y: GREEN.g,
            z: GREEN.b,
        };
        current_body_rgb.distance(Vec3 {
            x: color.r,
            y: color.g,
            z: color.b,
        }) < real_color_gap
            || current_body_rgb.distance(green_rgb) < real_color_gap
    }) {
        color = Color::from_rgba(
            gen_range(50, 250),
            gen_range(50, 250),
            gen_range(50, 250),
            255,
        )
    }

    spawn_body(
        bodies,
        Vec2 { x, y },
        get_with_deviation(AVERAGE_ENERGY, rng),
        get_with_deviation(AVERAGE_SPEED, rng),
        get_with_deviation(AVERAGE_VISION_DISTANCE, rng),
        eating_strategy,
        get_with_deviation(AVERAGE_DIVISION_THRESHOLD, rng),
        get_with_deviation(AVERAGE_MASS, rng),
        IQStage::Zero,
        color,
    );
}

fn window_conf() -> Conf {
    Conf {
        window_title: "eportal".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Make the window fullscreen for Linux
    // if OS == "linux" {
    set_fullscreen(true);
    sleep(Duration::from_secs(1));
    next_frame().await;
    // }

    let screen_size = (screen_width(), screen_height());
    let area_size = (screen_size.0 * OBJECT_RADIUS, screen_size.1 * OBJECT_RADIUS);

    let mut bodies: HashMap<usize, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: Vec<Plant> = Vec::with_capacity(PLANTS_N);

    let rng = &mut thread_rng();

    for i in 1..BODIES_N {
        let eating_strategy = if i >= BODY_EATERS_N {
            EatingStrategy::Plants
        } else {
            EatingStrategy::Bodies
        };

        randomly_spawn_body(&mut bodies, area_size, eating_strategy);
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

    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.0, area_size.1));
    default_camera(&mut camera, area_size);

    let mut zoom_mode = false;

    let mut last_updated = Instant::now();

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
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

        for (body_id, body) in bodies_to_iter {
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
            if is_draw_mode {
                draw_text(
                    &body_id.to_string(),
                    body.pos.x
                        - measure_text(&body_id.to_string(), None, BODY_INFO_FONT_SIZE, 1.0).width
                            / 2.0,
                    body.pos.y - OBJECT_RADIUS - MIN_GAP,
                    BODY_INFO_FONT_SIZE as f32,
                    WHITE,
                );

                draw_body!(body.pos, body.color);
                // draw_text(&format!("zoom {}", zoom), 10.0, 20.0, 20.0, WHITE);
            }
        }

        if is_draw_mode {
            draw_rectangle_lines(0.0, 0.0, area_size.0, area_size.1, 30.0, WHITE);
            for plant in &plants {
                draw_plant!(plant)
            }

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
