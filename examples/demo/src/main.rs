use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::{keyboard::Keycode, rect::Rect};

use image::{ImageBuffer, Rgb};
use opensimplex_noise_rs::OpenSimplexNoise;

const WIN_SIZE: [i32; 2] = [500, 500];

fn color_to_array(color: Color) -> [u8; 3] {
    [color.r, color.g, color.b]
}

fn get_id_from_pos(x: i32, y: i32) -> usize {
    (x + WIN_SIZE[0] * y) as usize
}

fn main() {
    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(WIN_SIZE[0] as u32, WIN_SIZE[1] as u32);

    let mut not_generated = true;

    let mut noise_map: Vec<f32> =vec![0.0; (WIN_SIZE[0] * WIN_SIZE[1]) as usize];

    let noise_generator = OpenSimplexNoise::new(Some(883_279_212_983_182_319));
    let scale = 0.044;

    for x in 0..WIN_SIZE[0] {
        for y in 0..WIN_SIZE[1] {
            let value = noise_generator.eval_2d(x as f64 * scale, y as f64 * scale) as f32;
            noise_map[get_id_from_pos(x, y)] = (value + 1.0) * 1.0 / 2.0;
        }
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "noise visualization demo",
            WIN_SIZE[0] as u32,
            WIN_SIZE[1] as u32,
        )
        .position_centered()
        .build()
        .unwrap();
        
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => { },
            }
        }

        for x in 0..WIN_SIZE[0] {
            for y in 0..WIN_SIZE[1] {
                let value = noise_map[get_id_from_pos(x, y)] * 255.0;

                let color = Color::RGB(value as u8, value as u8, value as u8);
                if not_generated {
                    let pixel = image.get_pixel_mut(x as u32, y as u32);
                    *pixel = image::Rgb(color_to_array(color));
                }

                canvas.set_draw_color(color);
                let _ = canvas.fill_rect::<_>(Rect::new(x as i32, y as i32, 1, 1));
            }
        }

        not_generated = false;

        canvas.present();
    }

    image.save("output.png").unwrap();
}
