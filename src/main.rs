extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate find_folder;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;

mod field;


pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    field: field::Field,
    mouse_pos: [f64; 2],
    window_sizes: [u32; 2],
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BG_COLOR: [f32; 4] = [0.6, 0.6, 0.6, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BG_COLOR, gl);
            self.field.render(&c, gl)
        });
    }

    fn button_release(&mut self, button: &Button) {
        self.field.mouse_release(self.mouse_pos[0], self.mouse_pos[1], button);
    }


    fn button_press(&mut self, button: &Button) {
        if button == &Button::Keyboard(Key::R) {
            self.field = field::Field::new(self.window_sizes, 16, 16)
        }

        self.field.mouse_press(self.mouse_pos[0], self.mouse_pos[1], button);
    }


    fn mouse_cursor(&mut self, pos: [f64; 2]) {
        self.mouse_pos = pos
    }
}

fn main() {
    let window_sizes: [u32; 2] = [520, 520];

    let opengl = OpenGL::V2_1;

    let mut window: Window = WindowSettings::new("Simple Minesweeper", window_sizes)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        field: field::Field::new(window_sizes, 16, 16),
        mouse_pos: [0.0, 0.0],
        window_sizes
    };

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }
        
        if let Some(button) = e.press_args() {
            app.button_press(&button);
        }

        if let Some(button) = e.release_args() {
            app.button_release(&button);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            app.mouse_cursor(pos);
        }
    }
}