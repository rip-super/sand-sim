use rand::RngExt;
use rand::rngs::SmallRng;
use rusty_console_game_engine::{color::*, key::*, prelude::*};

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

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Place,
    Delete,
}

struct SandSim {
    grid: Box<[Cell; WIDTH * HEIGHT]>,
    next_grid: Box<[Cell; WIDTH * HEIGHT]>,
    hue: f32,
    accumulator: f32,
    rng: SmallRng,
    tool: Tool,
}

impl SandSim {
    fn new() -> Self {
        Self {
            grid: Box::new([Cell::default(); WIDTH * HEIGHT]),
            next_grid: Box::new([Cell::default(); WIDTH * HEIGHT]),
            hue: 0.0,
            accumulator: 0.0,
            rng: rand::make_rng(),
            tool: Tool::Place,
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

        let left_to_right = self.rng.random_bool(0.5);

        for y in (0..HEIGHT).rev() {
            let xs: Vec<usize> = if left_to_right {
                (0..WIDTH).collect()
            } else {
                (0..WIDTH).rev().collect()
            };

            for x in xs {
                let idx = Self::get_idx(x, y);
                let cell = self.grid[idx];

                if !cell.filled {
                    continue;
                }

                let mut moved = false;
                let below_y = y + 1;

                if below_y < HEIGHT {
                    let below_idx = Self::get_idx(x, below_y);

                    if !self.next_grid[below_idx].filled {
                        self.next_grid[below_idx] = Cell {
                            filled: true,
                            color: cell.color,
                            vel: cell.vel + GRAVITY,
                        };
                        moved = true;
                    }

                    if !moved {
                        let dir = if self.rng.random_bool(0.5) { 1 } else { -1 };
                        for &dx in &[dir, -dir] {
                            let nx = x as i32 + dx;
                            if nx >= 0 && nx < WIDTH as i32 {
                                let diag_idx = Self::get_idx(nx as usize, below_y);

                                if !self.next_grid[diag_idx].filled {
                                    self.next_grid[diag_idx] = Cell {
                                        filled: true,
                                        color: cell.color,
                                        vel: cell.vel + GRAVITY,
                                    };
                                    moved = true;
                                    break;
                                }
                            }
                        }
                    }
                }

                if !moved {
                    self.next_grid[idx] = Cell {
                        filled: true,
                        color: cell.color,
                        vel: cell.vel + GRAVITY,
                    };
                }
            }
        }

        std::mem::swap(&mut self.grid, &mut self.next_grid);
    }

    fn handle_input(&mut self, engine: &mut ConsoleGameEngine<Self>, dt: f32) {
        if engine.key_pressed(ONE) {
            self.tool = Tool::Place;
        }
        if engine.key_pressed(TWO) {
            self.tool = Tool::Delete;
        }

        let mx = engine.mouse_x();
        let my = engine.mouse_y();

        if engine.mouse_held(LEFT) {
            for dy in -BRUSH_SIZE..=BRUSH_SIZE {
                for dx in -BRUSH_SIZE..=BRUSH_SIZE {
                    if dx * dx + dy * dy > BRUSH_SIZE * BRUSH_SIZE {
                        continue;
                    }

                    let nx = mx + dx;
                    let ny = my + dy;

                    if nx >= 0 && nx < WIDTH as i32 && ny >= 0 && ny < HEIGHT as i32 {
                        let idx = Self::get_idx(nx as usize, ny as usize);

                        match self.tool {
                            Tool::Place => {
                                if self.rng.random_bool(0.75) && !self.grid[idx].filled {
                                    self.grid[idx] = Cell {
                                        filled: true,
                                        color: self.next_color(),
                                        vel: 1.0,
                                    };
                                }
                            }
                            Tool::Delete => {
                                self.grid[idx] = Cell::default();
                            }
                        }
                    }
                }
            }

            if self.tool == Tool::Place {
                self.hue = (self.hue + 60.0 * dt) % 255.0;
            }
        }
    }

    fn draw_cursor(&self, engine: &mut ConsoleGameEngine<Self>) {
        let mx = engine.mouse_x();
        let my = engine.mouse_y();

        let color = match self.tool {
            Tool::Place => FG_WHITE,
            Tool::Delete => FG_RED,
        };

        engine.draw_circle_with(mx, my, BRUSH_SIZE + 1, SOLID, FG_DARK_GREY);
        engine.draw_circle_with(mx, my, BRUSH_SIZE, SOLID, color);
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

        self.draw_cursor(engine);

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
