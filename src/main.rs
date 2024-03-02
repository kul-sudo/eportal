mod constants;

use constants::*;

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use macroquad::{
    camera::{set_camera, Camera2D},
    color::{Color, BLACK, GREEN, WHITE},
    math::{vec2, Rect, Vec2},
    miniquad::window::set_fullscreen,
    rand::gen_range,
    shapes::{draw_circle, draw_line, draw_triangle},
    text::{draw_text, measure_text},
    window::{clear_background, next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, PartialEq, Clone, Copy)]
enum Preference {
    Bodies,
    Plants,
}

#[derive(Debug, Clone, Copy)]
enum Status<'a> {
    FollowedBy(&'a Body<'a>),
    FollowingBody(&'a Body<'a>),
    FollowingPlant(Plant),
    EscapingBody(&'a Body<'a>),
    Sleeping,
}

enum IQStage {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[derive(Debug, Clone, Copy)]
struct Body<'a> {
    id: usize,
    x: f32,
    y: f32,
    energy: f32,
    nearest_plant: Option<(f32, (usize, Plant))>,
    // nearest_body: Option<(f32, (usize, &'a Body<'a>))>,
    speed: f32,
    color: Color,
    preference: Preference,
    status: Status<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plant {
    x: f32,
    y: f32,
}

fn distance(components1: Vec<f32>, components2: Vec<f32>) -> f32 {
    f32::sqrt(
        components1
            .iter()
            .enumerate()
            .map(|(index, component)| (component - components2[index]).powf(2.0))
            .sum(),
    )
}

fn draw_body(x: f32, y: f32, color: Color) {
    draw_circle(x, y, OBJECT_RADIUS, color);
}

fn draw_plant(x: f32, y: f32) {
    draw_triangle(
        Vec2 {
            x,
            y: y - OBJECT_RADIUS,
        },
        Vec2 {
            x: x + OBJECT_RADIUS * (ROOT_OF_3_DIVIDED_BY_2),
            y: y + OBJECT_RADIUS / 2.0,
        },
        Vec2 {
            x: x - OBJECT_RADIUS * (ROOT_OF_3_DIVIDED_BY_2),
            y: y + OBJECT_RADIUS / 2.0,
        },
        GREEN,
    );
}

fn get_nearest_plant_for_body(plants: &[Plant], body: &Body) -> Option<(f32, (usize, Plant))> {
    let (plant_id, plant) = plants
        .iter()
        .enumerate()
        .min_by_key(|(_, plant)| distance(vec![plant.x, plant.y], vec![body.x, body.y]) as isize)?;
    Some((
        distance(vec![plant.x, plant.y], vec![body.x, body.y]),
        (plant_id, *plant),
    ))
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
    bodies: &mut [Body],
    plants: &mut Vec<Plant>,
    rng: &mut ThreadRng,
    area_size: (f32, f32),
) {
    let integer_area_size = (area_size.0 as u16, area_size.1 as u16);

    let starting_point = Instant::now();

    let mut x;
    let mut y;

    while {
        if starting_point.elapsed().as_nanos()
            >= Duration::from_millis(PLANT_SPAWN_TIME_LIMIT).as_nanos()
        {
            return;
        }
        x = rng.gen_range(0..integer_area_size.0) as f32;
        y = rng.gen_range(0..integer_area_size.1) as f32;
        (x <= OBJECT_RADIUS + MIN_GAP || x >= area_size.0 - OBJECT_RADIUS - MIN_GAP)
            || (y <= OBJECT_RADIUS + MIN_GAP || y >= area_size.1 - OBJECT_RADIUS - MIN_GAP)
            || bodies.iter().any(|body| {
                distance(vec![body.x, body.y], vec![x, y]) <= OBJECT_RADIUS * 2.0 + MIN_GAP
            })
            || plants.iter().any(|plant| {
                distance(vec![plant.x, plant.y], vec![x, y]) <= OBJECT_RADIUS * 2.0 + MIN_GAP
            })
    } {}

    plants.push(Plant { x, y });
}

fn spawn_body(bodies: &mut Vec<Body>, x: f32, y: f32, rng: &mut ThreadRng, color: Color) {
    bodies.push(Body {
        id: bodies.len() + 1,
        x,
        y,
        energy: 100.0,
        nearest_plant: None,
        // nearest_body: None,
        speed: rng.gen_range(0.1..5.5),
        color,
        preference: *[Preference::Plants, Preference::Bodies]
            .choose(rng)
            .unwrap(),
        status: Status::Sleeping,
    });
}

fn randomly_spawn_body(bodies: &mut Vec<Body>, rng: &mut ThreadRng, area_size: (f32, f32)) {
    let integer_area_size = (area_size.0 as u16, area_size.1 as u16);

    let mut x;
    let mut y;

    while {
        x = rng.gen_range(0..integer_area_size.0) as f32;
        y = rng.gen_range(0..integer_area_size.1) as f32;
        (x <= OBJECT_RADIUS + MIN_GAP || x >= area_size.0 - OBJECT_RADIUS - MIN_GAP)
            || (y <= OBJECT_RADIUS + MIN_GAP || y >= area_size.1 - OBJECT_RADIUS - MIN_GAP)
            || bodies.iter().any(|body| {
                distance(vec![body.x, body.y], vec![x, y]) < OBJECT_RADIUS * 2.0 + MIN_GAP
            })
    } {}

    let real_color_gap = COLOR_GAP / (BODIES_N as f32).powf(1.0 / 3.0);

    let mut color = Color::from_rgba(
        gen_range(50, 250),
        gen_range(50, 250),
        gen_range(50, 250),
        255,
    );

    while bodies.iter().any(|body| {
        distance(
            vec![body.color.r, body.color.g, body.color.b],
            vec![color.r, color.g, color.b],
        ) < real_color_gap
    }) {
        color = Color::from_rgba(
            gen_range(50, 250),
            gen_range(50, 250),
            gen_range(50, 250),
            255,
        )
    }

    spawn_body(bodies, x, y, rng, color);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "My game".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Make the window fullscreen
    set_fullscreen(true);
    sleep(Duration::from_secs(1));
    next_frame().await;

    let screen_size = (screen_width(), screen_height());
    let area_size = (screen_size.0 * OBJECT_RADIUS, screen_size.1 * OBJECT_RADIUS);

    let mut bodies: Vec<Body> = Vec::with_capacity(BODIES_N);
    let mut plants: Vec<Plant> = Vec::with_capacity(PLANTS_N);

    let rng = &mut thread_rng();

    // Spawn the bodies
    for _ in 0..BODIES_N {
        randomly_spawn_body(&mut bodies, rng, area_size)
    }

    // Spawn the plants
    for _ in 0..PLANTS_N {
        randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
    }

    // Get the nearest plant for each spawned body
    for body in bodies.iter_mut() {
        body.nearest_plant = get_nearest_plant_for_body(&plants, body);
        // body.nearest_body = get_nearest_body_for_body(&bodies, body)
    }

    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, area_size.0, area_size.1));
    let mut zoom = 1.0;

    loop {
        clear_background(BLACK);
        {
            camera.target = vec2(area_size.0 / 2.0, area_size.1 / 2.0);
            // println!("{:?}", camera.zoom);
            // println!("a = {:?}", 1.0 / area_size.0);
            camera.zoom = vec2(zoom * 2.0 / area_size.0, zoom * 2.0 / area_size.1);
            // camera.zoom = vec2(1.0 / area_size.0 * zoom, -1.0 / area_size.1 * zoom);
            set_camera(&camera);
        }

        // bodies.shuffle(rng);
        bodies.retain(|body| body.energy > 0.0);
        if rng.gen_range(0.0..1.0) > 1.0 - PLANT_SPAWN_CHANCE {
            randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
        }

        for (body_id_in_vec, body) in bodies.iter_mut().enumerate() {
            body.nearest_plant = get_nearest_plant_for_body(&plants, body);
            // body.nearest_body = get_nearest_body_for_body(&mut bodies, body);
            // update_nearest_body(body, &bodies);

            // body.energy -= ENERGY_FOR_WALKING;
            match body.preference {
                Preference::Plants => {
                    // Move towards the nearest plant
                    let (distance_to_plant, (plant_id, nearest_plant)) =
                        body.nearest_plant.unwrap();
                    let (dx, dy) = (nearest_plant.x - body.x, nearest_plant.y - body.y);
                    let coeff = body.speed / distance_to_plant;

                    body.x += coeff * dx;
                    body.y += coeff * dy;
                    body.status = Status::FollowingPlant(nearest_plant);

                    // draw_line(body.x, body.y, nearest_plant.x, nearest_plant.y, 5.0, WHITE);

                    // If there's been a contact between the body and a plant, handle it
                    if distance(vec![body.x, body.y], vec![nearest_plant.x, nearest_plant.y])
                        <= OBJECT_RADIUS
                    {
                        body.energy += PLANT_HP;
                        plants.remove(plant_id);
                        body.nearest_plant = None;
                        body.status = Status::Sleeping
                    }
                }
                Preference::Bodies => {
                    //     for (enemy_id, mut enemy_body) in bodies.clone().into_iter() {
                    //         if (enemy_body.x, enemy_body.y) == (body.x, body.y) {
                    //             bodies.remove(match body.hp.cmp(&enemy_body.hp) {
                    //                 Ordering::Less => {
                    //                     enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
                    //                     body_id
                    //                 }
                    //                 Ordering::Equal => {
                    //                     let options = [body_id, &enemy_id];
                    //                     let chosen = options.choose(rng).unwrap();
                    //
                    //                     if *chosen == body_id {
                    //                         enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
                    //                     } else {
                    //                         body.hp += enemy_body.hp.min(MAX_HP - body.hp);
                    //                     }
                    //
                    //                     chosen
                    //                 }
                    //                 Ordering::Greater => {
                    //                     enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
                    //                     &enemy_id
                    //                 }
                    //             });
                    //         }
                    //     }
                }
            }

            // draw_text(
            //     &body.id.to_string(),
            //     body.x - measure_text(&body.id.to_string(), None, FONT_SIZE, 1.0).width / 2.0,
            //     body.y - OBJECT_RADIUS - MIN_GAP,
            //     FONT_SIZE as f32,
            //     WHITE,
            // );
            //
            draw_body(body.x, body.y, body.color);
            for plant in plants.iter() {
                draw_plant(plant.x, plant.y)
            }
        }
        next_frame().await;
    }
}
