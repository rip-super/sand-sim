use rand::RngExt;
use rand::rngs::SmallRng;
use rusty_console_game_engine::{color::*, prelude::*};

const WIDTH: usize = 300;
const HEIGHT: usize = 200;
const BRUSH_SIZE: i32 = 3;
const GRAVITY: f32 = 0.2;
const TICK_RATE: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    filled: bool,
    color: u16,
    vel: f32,
}

struct SandSim {
    grid: Box<[Cell; WIDTH * HEIGHT]>,
    hue: f32,
    accumulator: f32,
    rng: SmallRng,
}

impl SandSim {
    fn new() -> Self {
        Self {
            grid: Box::new(
                [Cell {
                    filled: false,
                    color: FG_BLACK,
                    vel: 0.0,
                }; WIDTH * HEIGHT],
            ),
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
        for y in (0..HEIGHT - 1).rev() {
            for x in 0..WIDTH {
                let idx = Self::get_idx(x, y);
                let cell = self.grid[idx];

                if cell.filled {
                    let vel = cell.vel;
                    let new_y = (y as f32 + vel).min((HEIGHT - 1) as f32) as usize;

                    if new_y <= y {
                        self.grid[idx].vel += GRAVITY;
                        continue;
                    }

                    let mut moved = false;

                    for ty in (y + 1..=new_y).rev() {
                        let target_idx = Self::get_idx(x, ty);

                        if !self.grid[target_idx].filled {
                            self.grid[target_idx] = Cell {
                                filled: true,
                                color: cell.color,
                                vel: vel + GRAVITY,
                            };
                            self.grid[idx] = Cell {
                                filled: false,
                                color: FG_BLACK,
                                vel: 0.0,
                            };
                            moved = true;
                            break;
                        }

                        let dir = if self.rng.random_bool(0.5) {
                            -1i32
                        } else {
                            1i32
                        };
                        let dxs = [dir, -dir];

                        for &dx in dxs.iter() {
                            let nx = x as i32 + dx;
                            if nx >= 0 && nx < WIDTH as i32 {
                                let diag_idx = Self::get_idx(nx as usize, ty);
                                if !self.grid[diag_idx].filled {
                                    self.grid[diag_idx] = Cell {
                                        filled: true,
                                        color: cell.color,
                                        vel: vel + GRAVITY,
                                    };
                                    self.grid[idx] = Cell {
                                        filled: false,
                                        color: FG_BLACK,
                                        vel: 0.0,
                                    };
                                    moved = true;
                                    break;
                                }
                            }
                        }
                        if moved {
                            break;
                        }
                    }

                    if !moved {
                        self.grid[idx].vel += GRAVITY;
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, engine: &mut ConsoleGameEngine<Self>, dt: f32) {
        let mx = engine.mouse_x();
        let my = engine.mouse_y();

        if engine.mouse_held(LEFT) || engine.mouse_held(RIGHT) {
            for dy in -BRUSH_SIZE..=BRUSH_SIZE {
                for dx in -BRUSH_SIZE..=BRUSH_SIZE {
                    let nx = mx + dx;
                    let ny = my + dy;

                    if nx >= 0 && nx < WIDTH as i32 && ny >= 0 && ny < HEIGHT as i32 {
                        let idx = Self::get_idx(nx as usize, ny as usize);

                        if engine.mouse_held(LEFT) && self.rng.random_bool(0.75) {
                            if !self.grid[idx].filled {
                                self.grid[idx] = Cell {
                                    filled: true,
                                    color: self.next_color(),
                                    vel: 1.0,
                                };
                            }
                        } else if engine.mouse_held(RIGHT) {
                            self.grid[idx].filled = false;
                        }
                    }
                }
            }
        }

        self.hue = (self.hue + 60.0 * dt) % 255.0;
    }

    fn draw(&self, engine: &mut ConsoleGameEngine<Self>) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let cell = self.grid[Self::get_idx(x, y)];
                if cell.filled {
                    engine.draw_with(x as i32, y as i32, SOLID, cell.color);
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

        self.draw(engine);
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
