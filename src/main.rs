use std::io::Write;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::joystick::Joystick;
use std::time::Duration;
use sdl2::image::InitFlag;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics_framebuf::FrameBuf;
use embedded_snake::*;
use rand::rngs::ThreadRng;

pub const SCREEN_WIDTH: u32 = 128;
pub const SCREEN_HEIGHT: u32 = 128;
const PIXELS_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT * 2) as usize;
const BUFFER_SIZE: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

fn get_buffer(embedded_buffer: &mut [Rgb565; BUFFER_SIZE]) -> FrameBuf<Rgb565, &mut [Rgb565; BUFFER_SIZE]> {
    FrameBuf::new(embedded_buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
}

fn apply_buffer(pixels: &mut Vec<u8>, embedded_buffer: &mut [Rgb565; BUFFER_SIZE]) {
    for (i, mut chunk) in pixels.chunks_exact_mut(2).enumerate() {
        let color = embedded_buffer.get(i).unwrap();
        let value = RawU16::from(color.clone()).into_inner();
        let data: [u8; 2] = [(value & 0xFF) as u8, (value >> 8) as u8];
        chunk.write_all(&data).unwrap();
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();

    let available_joysticks = joystick_subsystem.num_joysticks().unwrap();

    let mut joystick: Option<Joystick> = None;
    for id in 0..available_joysticks {
        if let Ok(j) = joystick_subsystem.open(id) {
            joystick = Some(j);
            break;
        }
    }

    let mut game = SnakeGame::<100, Rgb565, ThreadRng>::new(
        128,
        128,
        3,
        3,
        rand::thread_rng(),
        Rgb565::RED,
        Rgb565::YELLOW,
        50,
    );

    let mut pixels: Vec<u8> = vec![0; PIXELS_SIZE];
    let mut embedded_buffer: [Rgb565; BUFFER_SIZE] = [Rgb565::BLACK; BUFFER_SIZE];

    let window = video_subsystem.window(
        "",
        SCREEN_WIDTH,
        SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_static(
        Some(PixelFormatEnum::RGB565), SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut direction = Direction::None;

    'running: loop {
        direction = Direction::None;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::JoyButtonDown { button_idx, .. } => {
                    direction = match button_idx {
                        13 => Direction::Up,
                        14 => Direction::Down,
                        15 => Direction::Left,
                        16 => Direction::Right,
                        _ => Direction::None,
                    };

                },
                _ => {},
            }

            game.set_direction(direction);
        }

        {
            let mut fbuf = get_buffer(&mut embedded_buffer);
            fbuf.clear(Rgb565::BLACK).unwrap();
            game.draw(&mut fbuf);
        }

        apply_buffer(&mut pixels, &mut embedded_buffer);
        texture.update(None, &pixels, (SCREEN_WIDTH * 2) as usize).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 20));
    }
}
