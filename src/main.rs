mod constants;

use constants::*;

use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use macroquad::{
    color::{Color, GREEN, WHITE},
    math::Vec2,
    miniquad::window::set_fullscreen,
    rand::gen_range,
    shapes::{draw_circle, draw_line, draw_triangle},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, PartialEq)]
enum Preference {
    Bodies,
    Plants,
}

#[derive(Debug, Clone, Copy)]
struct Body<'a> {
    x: f32,
    y: f32,
    energy: f32,
    nearest_plant: Option<(f32, Plant)>,
    speed: f32,
    color: Color,
    preference: &'a Preference,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plant {
    x: f32,
    y: f32,
}

macro_rules! id {
    () => {{
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }};
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
    draw_circle(x, y, BODY_RADIUS, color);
}

fn draw_plant(x: f32, y: f32) {
    draw_triangle(
        Vec2 {
            x,
            y: y - PLANT_RADIUS,
        },
        Vec2 {
            x: x + PLANT_RADIUS * (ROOT_OF_3_DIVIDED_BY_2),
            y: y + PLANT_RADIUS / 2.0,
        },
        Vec2 {
            x: x - PLANT_RADIUS * (ROOT_OF_3_DIVIDED_BY_2),
            y: y + PLANT_RADIUS / 2.0,
        },
        GREEN,
    );
}

fn get_closest_plant_for_body(
    plants: &HashMap<u128, Plant>,
    body: &mut Body,
) -> Option<(f32, Plant)> {
    let closest = *plants
        .values()
        .min_by_key(|plant| distance(vec![plant.x, plant.y], vec![body.x, body.y]) as isize)
        .unwrap();
    Some((
        distance(vec![closest.x, closest.y], vec![body.x, body.y]),
        closest,
    ))
}

fn update_nearest_plants(body: &mut Body, plants: HashMap<u128, Plant>) {
    body.nearest_plant = match body.nearest_plant {
        Some((current_distance, plant)) => {
            if !plants.values().collect::<Vec<_>>().contains(&&plant) {
                body.nearest_plant = None;
                get_closest_plant_for_body(&plants, body)
            } else {
                Some((
                    distance(vec![plant.x, plant.y], vec![body.x, body.y]).min(current_distance),
                    plant,
                ))
            }
        }
        None => get_closest_plant_for_body(&plants, body),
    };
}

fn randomly_spawn_plant(
    bodies: &mut HashMap<usize, Body>,
    plants: &mut HashMap<u128, Plant>,
    rng: &mut ThreadRng,
    area_size: (f32, f32),
) {
    let integer_area_size = (area_size.0 as u16, area_size.1 as u16);
    let (mut x, mut y) = (
        rng.gen_range(0..integer_area_size.0) as f32,
        rng.gen_range(0..integer_area_size.1) as f32,
    );

    while (x <= PLANT_RADIUS + MIN_GAP || x >= area_size.0 - PLANT_RADIUS - MIN_GAP)
        || (y <= PLANT_RADIUS + MIN_GAP || y >= area_size.1 - PLANT_RADIUS - MIN_GAP)
        || bodies.values().any(|body| {
            distance(vec![body.x, body.y], vec![x, y]) < BODY_RADIUS + PLANT_RADIUS + MIN_GAP
        })
        || plants.values().any(|plant| {
            distance(vec![plant.x, plant.y], vec![x, y]) < PLANT_RADIUS * 2.0 + MIN_GAP
        })
    {
        x = rng.gen_range(0..integer_area_size.0) as f32;
        y = rng.gen_range(0..integer_area_size.1) as f32;
    }

    plants.insert(id!(), Plant { x, y });
}

fn randomly_spawn_body(
    bodies: &mut HashMap<usize, Body>,
    plants: &mut HashMap<u128, Plant>,
    rng: &mut ThreadRng,
    area_size: (f32, f32),
    id: usize,
) {
    let integer_area_size = (area_size.0 as u16, area_size.1 as u16);
    let (mut x, mut y) = (
        rng.gen_range(0..integer_area_size.0) as f32,
        rng.gen_range(0..integer_area_size.1) as f32,
    );

    while (x <= BODY_RADIUS + MIN_GAP || x >= area_size.0 - BODY_RADIUS - MIN_GAP)
        || (y <= BODY_RADIUS + MIN_GAP || y >= area_size.1 - BODY_RADIUS - MIN_GAP)
        || bodies.values().any(|body| {
            distance(vec![body.x, body.y], vec![x, y]) < BODY_RADIUS + PLANT_RADIUS + MIN_GAP
        })
        || plants.values().any(|plant| {
            distance(vec![plant.x, plant.y], vec![x, y]) < BODY_RADIUS + PLANT_RADIUS + MIN_GAP
        })
    {
        x = rng.gen_range(0..integer_area_size.0) as f32;
        y = rng.gen_range(0..integer_area_size.1) as f32;
    }

    let real_color_gap = COLOR_GAP / (BODIES_N as f32).powf(1.0 / 3.0);

    let mut color = Color::from_rgba(
        gen_range(50, 250),
        gen_range(50, 250),
        gen_range(50, 250),
        255,
    );

    while bodies.values().any(|body| {
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

    bodies.insert(
        id,
        Body {
            x,
            y,
            energy: 100.0,
            nearest_plant: None,
            speed: 1.0,
            color,
            preference: [&Preference::Plants, &Preference::Bodies]
                .choose(rng)
                .unwrap(),
        },
    );
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
    sleep(Duration::from_secs(2));
    next_frame().await;

    let area_size = (screen_width(), screen_height());
    let mut bodies: HashMap<usize, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: HashMap<u128, Plant> = HashMap::with_capacity(PLANTS_N);

    let rng = &mut thread_rng();

    // Spawn the plants
    for _ in 0..PLANTS_N {
        randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
    }

    // Spawn the bodies
    for id in 0..BODIES_N {
        randomly_spawn_body(&mut bodies, &mut plants, rng, area_size, id)
    }

    // Get the nearest plant for each spawned body
    for body in bodies.values_mut() {
        update_nearest_plants(body, plants.clone())
    }

    loop {
        bodies.retain(|_, body| body.energy > 0.0);
        if rng.gen_range(0.0..1.0) > 0.95 {
            randomly_spawn_plant(&mut bodies, &mut plants, rng, area_size)
        }
        for (body_id, body) in bodies.iter_mut() {
            update_nearest_plants(body, plants.clone());

            match body.preference {
                Preference::Plants => {
                    body.energy -= ENERGY_FOR_WALKING;

                    // Move towards the nearest plant
                    let (distance_to_plant, plant) = body.nearest_plant.unwrap();
                    let (dx, dy) = (plant.x - body.x, plant.y - body.y);
                    let coeff = body.speed / distance_to_plant;

                    body.x += coeff * dx;
                    body.y += coeff * dy;

                    draw_line(body.x, body.y, plant.x, plant.y, 5.0, WHITE);

                    // If there's been a contact between the body and a plant, handle it
                    for (plant_id, plant) in plants.clone().iter() {
                        if distance(vec![body.x, body.y], vec![plant.x, plant.y]) <= BODY_RADIUS {
                            body.energy += PLANT_HP;
                            plants.remove(plant_id);
                            body.nearest_plant = None;
                        };
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

            draw_text(
                &body.energy.to_string(),
                body.x - measure_text(&body_id.to_string(), None, FONT_SIZE, 1.0).width / 2.0,
                body.y - BODY_RADIUS - MIN_GAP,
                FONT_SIZE as f32,
                WHITE,
            );

            draw_body(body.x, body.y, body.color);
            for plant in plants.values() {
                draw_plant(plant.x, plant.y)
            }
        }
        next_frame().await;
    }
}
