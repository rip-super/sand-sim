use rand::RngExt;
use rand::rngs::SmallRng;
use rusty_console_game_engine::{color::*, prelude::*};

const WIDTH: usize = 300;
const HEIGHT: usize = 200;
const BRUSH_SIZE: i32 = 3;
const GRAVITY: f32 = 0.1;
const TICK_RATE: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    filled: bool,
    color: u16,
    vel: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            filled: false,
            color: FG_BLACK,
            vel: 0.0,
        }
    }
}

struct SandSim {
    grid: Box<[Cell; WIDTH * HEIGHT]>,
    next_grid: Box<[Cell; WIDTH * HEIGHT]>,
    hue: f32,
    accumulator: f32,
    rng: SmallRng,
}

impl SandSim {
    fn new() -> Self {
        Self {
            grid: Box::new([Cell::default(); WIDTH * HEIGHT]),
            next_grid: Box::new([Cell::default(); WIDTH * HEIGHT]),
            hue: 0.0,
            accumulator: 0.0,
            rng: rand::make_rng(),
        }
    }

    #[inline(always)]
    fn get_idx(x: usize, y: usize) -> usize {
        y * WIDTH + x
    }

    fn next_color(&self) -> u16 {
        match self.hue as u8 {
            0..=50 => FG_RED,
            51..=100 => FG_YELLOW,
            101..=150 => FG_GREEN,
            151..=200 => FG_CYAN,
            201..=230 => FG_BLUE,
            _ => FG_MAGENTA,
        }
    }

    fn update_simulation(&mut self) {
        for cell in self.next_grid.iter_mut() {
            *cell = Cell::default();
        }

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = Self::get_idx(x, y);
                let cell = self.grid[idx];

                if cell.filled {
                    let mut moved = false;
                    let velocity = cell.vel;
                    let new_y = (y as f32 + velocity).floor() as usize;

                    'outer: for ty in (y + 1..=new_y.min(HEIGHT - 1)).rev() {
                        let dir = if self.rng.random_bool(0.5) { 1 } else { -1 };

                        let center_idx = Self::get_idx(x, ty);
                        let side_a_x = x as i32 + dir;
                        let side_b_x = x as i32 - dir;

                        if !self.grid[center_idx].filled {
                            self.next_grid[center_idx] = Cell {
                                filled: true,
                                color: cell.color,
                                vel: velocity + GRAVITY,
                            };
                            moved = true;
                            break 'outer;
                        }

                        if side_a_x >= 0 && side_a_x < WIDTH as i32 {
                            let idx_a = Self::get_idx(side_a_x as usize, ty);
                            if !self.grid[idx_a].filled {
                                self.next_grid[idx_a] = Cell {
                                    filled: true,
                                    color: cell.color,
                                    vel: velocity + GRAVITY,
                                };
                                moved = true;
                                break 'outer;
                            }
                        }

                        if side_b_x >= 0 && side_b_x < WIDTH as i32 {
                            let idx_b = Self::get_idx(side_b_x as usize, ty);
                            if !self.grid[idx_b].filled {
                                self.next_grid[idx_b] = Cell {
                                    filled: true,
                                    color: cell.color,
                                    vel: velocity + GRAVITY,
                                };
                                moved = true;
                                break 'outer;
                            }
                        }
                    }

                    if !moved && !self.next_grid[idx].filled {
                        self.next_grid[idx] = Cell {
                            filled: true,
                            color: cell.color,
                            vel: velocity + GRAVITY,
                        };
                    }
                }
            }
        }

        std::mem::swap(&mut self.grid, &mut self.next_grid);
    }

    fn handle_input(&mut self, engine: &mut ConsoleGameEngine<Self>, dt: f32) {
        let mx = engine.mouse_x();
        let my = engine.mouse_y();

        if engine.mouse_held(LEFT) {
            for dy in -BRUSH_SIZE..=BRUSH_SIZE {
                for dx in -BRUSH_SIZE..=BRUSH_SIZE {
                    let nx = mx + dx;
                    let ny = my + dy;

                    if nx >= 0
                        && nx < WIDTH as i32
                        && ny >= 0
                        && ny < HEIGHT as i32
                        && self.rng.random_bool(0.75)
                    {
                        let idx = Self::get_idx(nx as usize, ny as usize);
                        if !self.grid[idx].filled {
                            self.grid[idx] = Cell {
                                filled: true,
                                color: self.next_color(),
                                vel: 1.0,
                            };
                        }
                    }
                }
            }

            self.hue = (self.hue + 60.0 * dt) % 255.0;
        }

        if engine.mouse_held(RIGHT) {
            for dy in -BRUSH_SIZE..=BRUSH_SIZE {
                for dx in -BRUSH_SIZE..=BRUSH_SIZE {
                    let nx = mx + dx;
                    let ny = my + dy;
                    if nx >= 0 && nx < WIDTH as i32 && ny >= 0 && ny < HEIGHT as i32 {
                        let idx = Self::get_idx(nx as usize, ny as usize);
                        self.grid[idx] = Cell::default();
                    }
                }
            }
        }
    }
}

impl ConsoleGame for SandSim {
    fn app_name(&self) -> &str {
        "Falling Sand Simulation"
    }

    fn create(&mut self, _engine: &mut ConsoleGameEngine<Self>) -> bool {
        true
    }

    fn update(&mut self, engine: &mut ConsoleGameEngine<Self>, dt: f32) -> bool {
        engine.clear(FG_BLACK);
        self.handle_input(engine, dt);

        self.accumulator += dt;
        while self.accumulator >= TICK_RATE {
            self.update_simulation();
            self.accumulator -= TICK_RATE;
        }

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let cell = self.grid[Self::get_idx(x, y)];
                if cell.filled {
                    engine.draw_with(x as i32, y as i32, SOLID, cell.color);
                }
            }
        }
        true
    }
}

fn main() {
    let mut engine = ConsoleGameEngine::new(SandSim::new());
    engine
        .construct_console(WIDTH as i16, HEIGHT as i16, 4, 4)
        .unwrap();
    engine.start();
}
