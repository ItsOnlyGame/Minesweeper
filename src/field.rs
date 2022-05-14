extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::path::Path;

pub(crate) use graphics::Context;
use opengl_graphics::{Filter, GlGraphics, Texture, TextureSettings};
use piston::{Button, MouseButton};
use rand::Rng;


pub struct Field {
    screen_size: [u32; 2],
    width: u16,
    height: u16,

    tiles: Vec<Vec<Tile>>,

    texture: Texture,
    game_state: GameState,

    last_mine: [u16; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    bomb_value: u8,
    tile_state: TileState,
    has_bomb: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum TileState {
    Default = 0,
    Sweeped = 1,
    Flag = 2,
    Mine = 3,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum GameState {
    Running = 0,
    GameOver = 1,
}

impl Field {
    pub fn new(screen_size: [u32; 2], width: u16, height: u16) -> Field {
        let texture_settings = TextureSettings::new()
            .compress(false)
            .min(Filter::Nearest)
            .mag(Filter::Nearest);
        let texture = Texture::from_path(Path::new("./assets/2000.png"), &texture_settings).unwrap();

        let generated_field = generate_field(width as usize, height as usize);

        return Field {
            screen_size,
            width,
            height,
            tiles: generated_field,
            texture,
            game_state: GameState::Running,
            last_mine: [0, 0],
        };
    }

    pub fn mouse_release(&mut self, mut m_x: f64, mut m_y: f64, button: &Button) {
        if self.game_state == GameState::GameOver {
            return;
        }

        let offset = [
            ((self.screen_size[0] - self.width as u32 * 32) / 2) as f64,
            ((self.screen_size[1] - self.height as u32 * 32) / 2) as f64,
        ];

        m_x = m_x - offset[0];
        m_y = m_y - offset[1];

        let x_pos = (m_x / 32 as f64) as i16;
        let y_pos = (m_y / 32 as f64) as i16;

        if outof_bounds_check(x_pos, y_pos, self.width as i16, self.height as i16) {
            return;
        }

        if button == &Button::Mouse(MouseButton::Left) {
            self.reveal_tile(x_pos as i16, y_pos as i16);
        }
    }

    pub fn mouse_press(&mut self, mut m_x: f64, mut m_y: f64, button: &Button) {
        if self.game_state == GameState::GameOver {
            return;
        }

        // Depending on the window resolution and the grid size, it may require an offset to be in the center
        let offset = [
            ((self.screen_size[0] - self.width as u32 * 32) / 2) as f64,
            ((self.screen_size[1] - self.height as u32 * 32) / 2) as f64,
        ];

        m_x = m_x - offset[0];
        m_y = m_y - offset[1];

        let x_pos = (m_x / 32 as f64) as i16;
        let y_pos = (m_y / 32 as f64) as i16;

        if outof_bounds_check(x_pos, y_pos, self.width as i16, self.height as i16) {
            return;
        }

        if button == &Button::Mouse(MouseButton::Right) {
            self.flag_tile(x_pos as i16, y_pos as i16);
        }
    }

    fn reveal_tile(&mut self, x: i16, y: i16) {
        let tile: &mut Tile = &mut self.tiles[x as usize][y as usize];

        if tile.tile_state == TileState::Sweeped {
            return;
        }

        if tile.tile_state == TileState::Flag {
            return;
        }

        if tile.has_bomb == true {
            tile.tile_state = TileState::Mine;
            if self.game_state == GameState::Running {
                self.game_state = GameState::GameOver;
                self.last_mine = [x as u16, y as u16];
                self.reveal_all();
            }

            return;
        }

        tile.tile_state = TileState::Sweeped;

        if tile.bomb_value == 0 && !tile.has_bomb {
            for x_offset in -1..2 {
                for y_offset in -1..2 {
                    if x_offset == 0 && y_offset == 0 {
                        continue;
                    }
                    if outof_bounds_check(
                        x + x_offset as i16,
                        y + y_offset as i16,
                        self.width as i16,
                        self.height as i16,
                    ) {
                        continue;
                    }

                    self.reveal_tile(x + x_offset as i16, y + y_offset as i16);
                }
            }
        }
    }

    fn flag_tile(&mut self, x: i16, y: i16) {
        let tile: &mut Tile = &mut self.tiles[x as usize][y as usize];

        if tile.tile_state == TileState::Default {
            self.tiles[x as usize][y as usize].tile_state = TileState::Flag;
        } else if tile.tile_state == TileState::Flag {
            self.tiles[x as usize][y as usize].tile_state = TileState::Default;
        }
    }

    pub fn render(&mut self, c: &Context, gl: &mut GlGraphics) {
        use graphics::*;

        let translation: [f64; 2] = [
            ((self.screen_size[0] - self.width as u32 * 32) / 2) as f64,
            ((self.screen_size[1] - self.height as u32 * 32) / 2) as f64,
        ];

        for x_coord in 0..self.width as usize {
            for y_coord in 0..self.height as usize {
                let transform = c
                    .transform
                    .trans(translation[0], translation[1])
                    .trans((x_coord * 32) as f64, (y_coord * 32) as f64);

                let tile: &Tile = &self.tiles[x_coord][y_coord];

                if self.game_state == GameState::GameOver {
                    if self.last_mine[0] == x_coord as u16 && self.last_mine[1] == y_coord as u16 {
                        let image = Image::new()
                            .src_rect([6.0 * 16.0, 0.0, 16.0, 16.0])
                            .rect([0.0, 0.0, 32.0, 32.0]);
    
                        image.draw(&self.texture, &DrawState::default(), transform, gl);
    
                        continue;
                    }
                }

                match tile.tile_state {
                    TileState::Default => {
                        let image = Image::new()
                            .src_rect([0.0, 0.0, 16.0, 16.0])
                            .rect([0.0, 0.0, 32.0, 32.0]);
                        image.draw(&self.texture, &DrawState::default(), transform, gl);
                    }

                    TileState::Sweeped => {
                        if tile.bomb_value == 0 {
                            let image = Image::new()
                                .src_rect([16.0, 0.0, 16.0, 16.0])
                                .rect([0.0, 0.0, 32.0, 32.0]);
                            image.draw(&self.texture, &DrawState::default(), transform, gl);
                        } else {
                            let image = Image::new()
                                .src_rect([(tile.bomb_value - 1) as f64 * 16.0, 16.0, 16.0, 16.0])
                                .rect([0.0, 0.0, 32.0, 32.0]);
                            image.draw(&self.texture, &DrawState::default(), transform, gl);
                        }
                    }

                    TileState::Flag => {
                        let image = Image::new()
                            .src_rect([32.0, 0.0, 16.0, 16.0])
                            .rect([0.0, 0.0, 32.0, 32.0]);
                        image.draw(&self.texture, &DrawState::default(), transform, gl);
                    }

                    TileState::Mine => {
                        let image = Image::new()
                            .src_rect([5.0 * 16.0, 0.0, 16.0, 16.0])
                            .rect([0.0, 0.0, 32.0, 32.0]);
                        image.draw(&self.texture, &DrawState::default(), transform, gl);
                    }
                }
            }
        }
    }

    fn reveal_all(&mut self) {
        for x_coord in 0..self.width as usize {
            for y_coord in 0..self.height as usize {
                self.reveal_tile(x_coord as i16, y_coord as i16);
            }
        }
    }
}

fn generate_field(width: usize, height: usize) -> Vec<Vec<Tile>> {
    let mut tiles: Vec<Vec<Tile>> = Vec::with_capacity(width);

    for x in 0..width {
        tiles.push(Vec::with_capacity(height));

        for _y in 0..height {
            tiles[x].push(Tile {
                tile_state: TileState::Default,
                bomb_value: 0,
                has_bomb: rand::thread_rng().gen_bool(1.0 / 4.0),
            })
        }
    }

    for x in 0..width as i16 {
        for y in 0..height as i16 {
            let mut count: u8 = 0;
            for x_offset in -1..2 {
                for y_offset in -1..2 {
                    if x_offset == 0 && y_offset == 0 {
                        continue;
                    }

                    if outof_bounds_check(
                        x + x_offset as i16,
                        y + y_offset as i16,
                        width as i16,
                        height as i16,
                    ) {
                        continue;
                    }

                    if tiles[(x + x_offset) as usize][(y + y_offset) as usize].has_bomb == true {
                        count += 1;
                    }
                }
            }
            tiles[x as usize][y as usize].bomb_value = count;
        }
    }

    return tiles;
}

fn outof_bounds_check(x: i16, y: i16, width: i16, height: i16) -> bool {
    if x < 0 || x >= width as i16 {
        return true;
    }

    if y < 0 || y >= height as i16 {
        return true;
    }

    return false;
}
