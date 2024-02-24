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
    shapes::{draw_circle, draw_triangle},
    text::{draw_text, measure_text},
    window::{next_frame, screen_height, screen_width, Conf},
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

static BODIES_N: usize = 100;
static PLANTS_N: usize = 100;
static MAX_HP: u8 = 100;
static PLANT_HP: u8 = 25;

static BODY_RADIUS: f32 = 10.0;
static PLANT_RADIUS: f32 = 10.0;
static MIN_GAP: f32 = 3.0;
static COLOR_GAP: f32 = 0.7;

static FONT_SIZE: u16 = 17;

static ROOT_OF_3_DIVIDED_BY_2: f32 = 0.8660254;

#[derive(Debug)]
enum Preference {
    Bodies,
    Plants,
}

#[derive(Debug, Clone, Copy)]
struct Body<'a> {
    x: f32,
    y: f32,
    hp: u8,
    color: Color,
    preference: &'a Preference,
}

#[derive(Debug, Clone, Copy)]
struct Plant {
    x: f32,
    y: f32,
}

macro_rules! id {
    () => {{
        let full = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
        full[full.len() - 6..].to_string()
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

fn randomly_place_plant(
    bodies: &HashMap<String, Body>,
    plants: &mut HashMap<String, Plant>,
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
        || plants.values().any(|plant| {
            distance(vec![plant.x, plant.y], vec![x, y]) < PLANT_RADIUS * 2.0 + MIN_GAP
        })
    {
        x = rng.gen_range(0..integer_area_size.0) as f32;
        y = rng.gen_range(0..integer_area_size.1) as f32;
    }

    plants.insert(id!(), Plant { x, y });
}

fn randomly_place_body(
    bodies: &mut HashMap<String, Body>,
    plants: &mut HashMap<String, Plant>,
    rng: &mut ThreadRng,
    area_size: (f32, f32),
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
        gen_range(220, 250),
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
            gen_range(220, 250),
        )
    }

    bodies.insert(
        id!(),
        Body {
            x,
            y,
            hp: 100,
            color,
            preference: [&Preference::Bodies, &Preference::Plants]
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
    set_fullscreen(true);
    sleep(Duration::from_secs(2));
    next_frame().await;
    let area_size = (screen_width(), screen_height());

    let mut bodies: HashMap<String, Body> = HashMap::with_capacity(BODIES_N);
    let mut plants: HashMap<String, Plant> = HashMap::with_capacity(PLANTS_N);

    let rng = &mut thread_rng();

    for _ in 0..PLANTS_N {
        randomly_place_plant(&bodies, &mut plants, rng, area_size)
    }

    for _ in 0..BODIES_N {
        randomly_place_body(&mut bodies, &mut plants, rng, area_size)
    }

    // for plant in plants.clone().values() {
    //     draw_plant(plant.x, plant.y)
    // }

    next_frame().await;
    // loop {}

    loop {
        for (body_id, body) in bodies.clone().iter_mut() {
            // body.x = if random::<bool>() {
            //     if body.x != area_size.0 {
            //         body.x + 1.0
            //     } else {
            //         body.x
            //     }
            // } else {
            //     body.x.saturating_sub(1)
            // };
            //
            // body.y = if random::<bool>() {
            //     if body.y != area_size.1 {
            //         body.y + 1
            //     } else {
            //         body.y
            //     }
            // } else {
            //     body.y.saturating_sub(1)
            // };
            //
            // match body.preference {
            //     Preference::Plants => {
            //         for (plant_id, plant) in plants.clone().into_iter() {
            //             if (plant.x, plant.y) == (body.x, body.y) {
            //                 body.hp += PLANT_HP.min(MAX_HP - body.hp);
            //                 plants.remove(&plant_id);
            //                 for _ in 0..2 {
            //                     randomly_place_plant(&bodies, &mut plants, rng, area_size)
            //                 }
            //             }
            //         }
            //     }
            //     Preference::Bodies => {
            //         for (enemy_id, mut enemy_body) in bodies.clone().into_iter() {
            //             if (enemy_body.x, enemy_body.y) == (body.x, body.y) {
            //                 bodies.remove(match body.hp.cmp(&enemy_body.hp) {
            //                     Ordering::Less => {
            //                         enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //                         body_id
            //                     }
            //                     Ordering::Equal => {
            //                         let options = [body_id, &enemy_id];
            //                         let chosen = options.choose(rng).unwrap();
            //
            //                         if *chosen == body_id {
            //                             enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //                         } else {
            //                             body.hp += enemy_body.hp.min(MAX_HP - body.hp);
            //                         }
            //
            //                         chosen
            //                     }
            //                     Ordering::Greater => {
            //                         enemy_body.hp += body.hp.min(MAX_HP - enemy_body.hp);
            //                         &enemy_id
            //                     }
            //                 });
            //             }
            //         }
            //     }
            // }
            draw_text(
                &body_id.to_string(),
                body.x - measure_text(&body_id.to_string(), None, FONT_SIZE, 1.0).width / 2.0,
                body.y - BODY_RADIUS - MIN_GAP,
                FONT_SIZE as f32,
                WHITE,
            );
            draw_body(body.x, body.y, body.color);
            for plant in plants.clone().values() {
                draw_plant(plant.x, plant.y)
            }
        }
        // for _ in plants.clone().iter() {
        // randomly_place_plant(&bodies, &mut plants, rng, area_size);
        // }
        next_frame().await;
    }
}
