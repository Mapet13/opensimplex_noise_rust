use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent};
use piston::window::WindowSettings;

use opensimplex_noise_rs::OpenSimplexNoise;

const WIN_SIZE: [i32; 2] = [400, 400];

fn main() {
    let opengl = OpenGL::V3_2;

    let noise_generator = OpenSimplexNoise::new(Some(883_279_212_983_182_319));
    let scale = 0.044;

    let mut window: Window = WindowSettings::new("noise 3D visualization", [400, 400])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);
    let mut time = 0.0;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            use graphics::*;

            const BACKGROUND: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
            let square = rectangle::square(0.0, 0.0, 1.0);

            gl.draw(args.viewport(), |c, gl| {
                clear(BACKGROUND, gl);

                for x in 0..WIN_SIZE[0] {
                    for y in 0..WIN_SIZE[1] {
                        let mut value =
                            noise_generator.eval_3d(x as f64 * scale, y as f64 * scale, time)
                                as f32;
                        value = (value + 1.0) * 1.0 / 2.0;

                        let transform = c.transform.trans(x as f64, y as f64);

                        rectangle([value, value, value, 1.0], square, transform, gl);
                    }
                }
                time += 0.1;
            });
        }
    }
}
