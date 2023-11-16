use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use chrono::prelude::*;
use sdl2::render::{Texture, WindowCanvas};
use log::debug;

// config
const WIDTH: i16 = 1024;
const HEIGHT: i16 = 600;
const BIG_WIDTH: u32 = 96;
const BIG_HEIGHT: u32 = 96;
const SMALL_WIDTH: u32 = 48;
const SMALL_HEIGHT: u32 = 48;

// textures will be packed into binary
static ASSET_BIG_RED: &[u8] = include_bytes!("../assets/big_red.jpg");
static ASSET_BIG_GREEN: &[u8] = include_bytes!("../assets/big_green.jpg");
static ASSET_BIG_GREY: &[u8] = include_bytes!("../assets/big_grey.jpg");
static ASSET_SMALL_RED: &[u8] = include_bytes!("../assets/small_red.jpg");
static ASSET_SMALL_GREEN: &[u8] = include_bytes!("../assets/small_green.jpg");
static ASSET_SMALL_YELLOW: &[u8] = include_bytes!("../assets/small_yellow.jpg");
static ASSET_SMALL_GREY: &[u8] = include_bytes!("../assets/small_grey.jpg");

// will hold positions calculated once at startup
struct Positions {
    five_hours: [(i32, i32); 4],
    one_hours: [(i32, i32); 4],
    five_minutes: [(i32, i32); 11],
    one_minutes: [(i32, i32); 4],
    second: (i32, i32),
}

pub fn main() {
    env_logger::init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("c2k", WIDTH as u32, HEIGHT as u32)
        .borderless()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    debug!("canvas cleared");

    // load textures
    let texture_creator = canvas.texture_creator();
    let texture_big_red = texture_creator.load_texture_bytes(ASSET_BIG_RED).unwrap();
    let texture_big_green = texture_creator.load_texture_bytes(ASSET_BIG_GREEN).unwrap();
    let texture_big_grey = texture_creator.load_texture_bytes(ASSET_BIG_GREY).unwrap();
    let texture_small_red = texture_creator.load_texture_bytes(ASSET_SMALL_RED).unwrap();
    let texture_small_green = texture_creator.load_texture_bytes(ASSET_SMALL_GREEN).unwrap();
    let texture_small_yellow = texture_creator.load_texture_bytes(ASSET_SMALL_YELLOW).unwrap();
    let texture_small_grey = texture_creator.load_texture_bytes(ASSET_SMALL_GREY).unwrap();

    // calculate positions
    let positions = calculate_positions_v2();

    // start ticker
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    thread::spawn(move || {
        let mut last_state = Local::now().second() % 2;
        loop {
            if Local::now().second() % 2 != last_state {
                tx.send(1).unwrap();
                last_state = Local::now().second() % 2;
            }
            thread::sleep(Duration::new(0, 500_000_000u32));
        }
    });

    let mut second_counter = 0;
    let mut minute_cache = 60;
    let mut hour_cache = 24;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        let local: DateTime<Local> = Local::now();

        if minute_cache != local.minute() || second_counter < 3 {
            if minute_cache != local.minute() {
                second_counter = 0;
                debug!("second counter reset!");
            }
            update_minutes(&mut canvas, &texture_small_yellow, &texture_small_red, &texture_small_grey, &positions, local.minute());
            minute_cache = local.minute();
        }

        if hour_cache != local.hour() || second_counter < 3 {
            update_hours(&mut canvas, &texture_big_red, &texture_big_green, &texture_big_grey, &positions, local.hour());
            hour_cache = local.hour();
        }

        update_second(&mut canvas, &texture_small_green, &texture_small_grey, &positions, local.second());
        canvas.present();

        if second_counter < 3 {
            second_counter += 1;
        }
        debug!("second_counter: {}", second_counter);

        _ = rx.recv();
    }
}

fn update_second(canvas: &mut WindowCanvas, texture_green: &Texture, texture_grey: &Texture, positions: &Positions, seconds: u32) {
    let texture = if seconds % 2 == 0 {
        texture_grey
    } else {
        texture_green
    };
    canvas.copy(texture, None, Some(Rect::new(positions.second.0, positions.second.1, SMALL_WIDTH, SMALL_HEIGHT))).expect("render failed");
    debug!("seconds: {}", seconds);
}

fn update_minutes(canvas: &mut WindowCanvas, texture_yellow: &Texture, texture_red: &Texture, texture_grey: &Texture, positions: &Positions, minutes: u32) {
    let five_minutes = minutes / 5;
    let one_minutes = minutes % 5;

    let mut i = 1;
    for pos in positions.five_minutes.iter() {
        let texture = if i <= five_minutes {
            texture_yellow
        } else {
            texture_grey
        };
        canvas.copy(texture, None, Some(Rect::new(pos.0, pos.1, SMALL_WIDTH, SMALL_HEIGHT))).expect("render failed");
        i += 1;
    }

    i = 1;
    for pos in positions.one_minutes.iter() {
        let texture = if i <= one_minutes {
            texture_red
        } else {
            texture_grey
        };
        canvas.copy(texture, None, Some(Rect::new(pos.0, pos.1, SMALL_WIDTH, SMALL_HEIGHT))).expect("render failed");
        i += 1;
    }
    debug!("minutes: {}", minutes);
}

fn update_hours(canvas: &mut WindowCanvas, texture_red: &Texture, texture_green: &Texture, texture_grey: &Texture, positions: &Positions, hours: u32) {
    let five_hours = hours / 5;
    let one_hours = hours % 5;

    let mut i = 1;
    for pos in positions.five_hours.iter() {
        let texture = if i <= five_hours {
            texture_red
        } else {
            texture_grey
        };
        canvas.copy(texture, None, Some(Rect::new(pos.0, pos.1, BIG_WIDTH, BIG_HEIGHT))).expect("render failed");
        i += 1;
    }

    i = 1;
    for pos in positions.one_hours.iter() {
        let texture = if i <= one_hours {
            texture_green
        } else {
            texture_grey
        };
        canvas.copy(texture, None, Some(Rect::new(pos.0, pos.1, BIG_WIDTH, BIG_HEIGHT))).expect("render failed");
        i += 1;
    }
    debug!("hours: {}", hours);
}

fn calculate_positions_v2() -> Positions {
    let hours_padding_x = 100.0;
    let minutes_padding_x = 200.0;
    let five_minutes_padding_x = 50.0;
    let height_offset = HEIGHT as f32 / 6.0;
    let hours_width_offset = (WIDTH as f32 - (hours_padding_x * 2.0)) / 3.0;
    let minutes_width_offset = (WIDTH as f32 - (minutes_padding_x * 2.0)) / 3.0;
    let five_minutes_width_offset = (WIDTH as f32 - (five_minutes_padding_x * 2.0)) / 10.0;

    let five_hours: [(i32, i32); 4] = [
        (
            (BIG_WIDTH as f32 / -2.0 + hours_padding_x) as i32,
            0
        ), (
            (BIG_WIDTH as f32 / -2.0 + hours_padding_x + hours_width_offset) as i32,
            0
        ), (
            (BIG_WIDTH as f32 / -2.0 + hours_padding_x + hours_width_offset * 2.0) as i32,
            0
        ), (
            ((BIG_WIDTH as f32 / -2.0) + (WIDTH as f32 - hours_padding_x)) as i32,
            0
        )];

    let one_hours = [
        (
            (BIG_WIDTH as f32 / -2.0 + hours_padding_x) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        ), (
            (BIG_WIDTH as f32 / -2.0 + hours_padding_x + hours_width_offset) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        ), (
            (BIG_WIDTH as f32 / -2.0 + hours_padding_x + hours_width_offset * 2.0) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        ), (
            ((BIG_WIDTH as f32 / -2.0) + (WIDTH as f32 - hours_padding_x)) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        )];

    let mut five_minutes = [(0, 0); 11];
    #[allow(unused_assignments)]
    let mut addend = five_minutes_padding_x;
    for i in 0..11 {
        if i == 10 {
            addend = WIDTH as f32 - five_minutes_padding_x;
        } else {
            addend = five_minutes_padding_x + (five_minutes_width_offset * (i as f32));
        }
        five_minutes[i] = (
            ((SMALL_WIDTH as f32 / -2.0) + addend) as i32,
            height_offset as i32 * 3 + (i as i32 % 2 * SMALL_HEIGHT as i32) - (SMALL_HEIGHT / 4) as i32
        );
    }

    let one_minutes = [
        (
            (SMALL_WIDTH as f32 / -2.0 + minutes_padding_x) as i32,
            HEIGHT as i32 - SMALL_HEIGHT as i32 * 3
        ), (
            (SMALL_WIDTH as f32 / -2.0 + minutes_padding_x + minutes_width_offset) as i32,
            HEIGHT as i32 - SMALL_HEIGHT as i32 * 3
        ), (
            (SMALL_WIDTH as f32 / -2.0 + minutes_padding_x + minutes_width_offset * 2.0) as i32,
            HEIGHT as i32 - SMALL_HEIGHT as i32 * 3
        ), (
            ((SMALL_WIDTH as f32 / -2.0) + (WIDTH as f32 - minutes_padding_x)) as i32,
            HEIGHT as i32 - SMALL_HEIGHT as i32 * 3
        )];

    let second_x: i32 = (WIDTH as f32 / 2.0 - SMALL_WIDTH as f32 / 2.0) as i32;
    let second_y: i32 = HEIGHT as i32 - SMALL_HEIGHT as i32;

    Positions {
        five_hours,
        one_hours,
        five_minutes,
        one_minutes,
        second: (second_x, second_y),
    }
}


/*fn calculate_positions_v1() -> Positions {
    let height_offset = HEIGHT / 6;
    let big_width_offset = WIDTH / 5;
    let small_width_offset = WIDTH / 12;

    let five_hours = [
        (
            big_width_offset as i32 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 - (BIG_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 2 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 - (BIG_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 3 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 - (BIG_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 4 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 - (BIG_HEIGHT / 2) as i32
        )];

    let one_hours = [
        (
            big_width_offset as i32  - (BIG_WIDTH / 2) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 2 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 3 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 4 - (BIG_WIDTH / 2) as i32,
            height_offset as i32 * 2 - (BIG_HEIGHT / 2) as i32
        )];

    let mut five_minutes = [(0, 0); 11];
    for i in 0..11 {
        five_minutes[i] = (
            small_width_offset as i32 * (i as i32 + 1) - (SMALL_WIDTH / 2) as i32,
            height_offset as i32 * 3 + (i as i32 % 2 * SMALL_HEIGHT as i32) - (SMALL_HEIGHT / 2) as i32
        );
    }

    let one_minutes = [
        (
            big_width_offset as i32 - (SMALL_WIDTH / 2) as i32,
            height_offset as i32 * 4 - (SMALL_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 2 - (SMALL_WIDTH / 2) as i32,
            height_offset as i32 * 4 - (SMALL_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 3 - (SMALL_WIDTH / 2) as i32,
            height_offset as i32 * 4 - (SMALL_HEIGHT / 2) as i32
        ), (
            big_width_offset as i32 * 4 - (SMALL_WIDTH / 2) as i32,
            height_offset as i32 * 4 - (SMALL_HEIGHT / 2) as i32
        )];

    let second_x: i32 = WIDTH as i32 / 2 - SMALL_WIDTH as i32 / 2;
    let second_y: i32 = height_offset as i32 * 5 - SMALL_HEIGHT as i32 / 2;

    Positions {
        five_hours: five_hours,
        one_hours: one_hours,
        five_minutes: five_minutes,
        one_minutes: one_minutes,
        second: (second_x, second_y),
    }
}*/
