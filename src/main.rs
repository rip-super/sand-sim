use rusty_console_game_engine::{color::*, prelude::*};
use std::{thread, time::Duration};

const WIDTH: usize = 120;
const HEIGHT: usize = 80;
const BRUSH_SIZE: i32 = 3;

const GRAVITY: f32 = 0.2;

#[derive(Clone, Copy)]
struct Cell {
    filled: bool,
    color: u16,
}

struct SandSim {
    grid: [[Cell; WIDTH]; HEIGHT],
    velocity: [[f32; WIDTH]; HEIGHT],
    hue: u8,
}

impl SandSim {
    fn new() -> Self {
        Self {
            grid: [[Cell {
                filled: false,
                color: FG_BLACK,
            }; WIDTH]; HEIGHT],
            velocity: [[0.0; WIDTH]; HEIGHT],
            hue: 0,
        }
    }

    fn next_color(&self) -> u16 {
        match self.hue {
            0..=50 => FG_RED,
            51..=100 => FG_YELLOW,
            101..=150 => FG_GREEN,
            151..=200 => FG_CYAN,
            201..=230 => FG_BLUE,
            _ => FG_MAGENTA,
        }
    }

    fn update_simulation(&mut self) {
        let mut next_grid = [[Cell {
            filled: false,
            color: FG_BLACK,
        }; WIDTH]; HEIGHT];
        let mut next_velocity = [[0.0; WIDTH]; HEIGHT];

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if self.grid[y][x].filled {
                    let vel = self.velocity[y][x];
                    let new_y = (y as f32 + vel) as i32;

                    let mut moved = false;

                    for ty in ((y as i32 + 1)..=new_y.min((HEIGHT - 1) as i32)).rev() {
                        let dir = if rand::random::<bool>() { -1 } else { 1 };

                        let below = self.grid[ty as usize][x].filled;

                        let below_left = if x > 0 {
                            self.grid[ty as usize][x - 1].filled
                        } else {
                            true
                        };

                        let below_right = if x < WIDTH - 1 {
                            self.grid[ty as usize][x + 1].filled
                        } else {
                            true
                        };

                        if !below {
                            next_grid[ty as usize][x] = self.grid[y][x];
                            next_velocity[ty as usize][x] = vel + GRAVITY;
                            moved = true;
                            break;
                        } else if dir == -1 && !below_left {
                            next_grid[ty as usize][x - 1] = self.grid[y][x];
                            next_velocity[ty as usize][x - 1] = vel + GRAVITY;
                            moved = true;
                            break;
                        } else if dir == 1 && !below_right {
                            next_grid[ty as usize][x + 1] = self.grid[y][x];
                            next_velocity[ty as usize][x + 1] = vel + GRAVITY;
                            moved = true;
                            break;
                        }
                    }

                    if !moved {
                        next_grid[y][x] = self.grid[y][x];
                        next_velocity[y][x] = vel + GRAVITY;
                    }
                }
            }
        }

        self.grid = next_grid;
        self.velocity = next_velocity;
    }

    fn handle_input(&mut self, engine: &mut ConsoleGameEngine<Self>) {
        let mx = engine.mouse_x();
        let my = engine.mouse_y();

        for dy in -BRUSH_SIZE..=BRUSH_SIZE {
            for dx in -BRUSH_SIZE..=BRUSH_SIZE {
                let nx = mx + dx;
                let ny = my + dy;

                if nx >= 0 && nx < WIDTH as i32 && ny >= 0 && ny < HEIGHT as i32 {
                    let x = nx as usize;
                    let y = ny as usize;

                    if engine.mouse_held(LEFT) && rand::random::<f32>() < 0.75 {
                        self.grid[y][x].filled = true;
                        self.grid[y][x].color = self.next_color();
                        self.velocity[y][x] = 1.0;
                    }

                    if engine.mouse_held(RIGHT) {
                        self.grid[y][x].filled = false;
                        self.velocity[y][x] = 0.0;
                    }
                }
            }
        }

        self.hue = self.hue.wrapping_add(1);
    }

    fn draw(&self, engine: &mut ConsoleGameEngine<Self>) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.grid[y][x].filled {
                    engine.draw_with(x as i32, y as i32, SOLID, self.grid[y][x].color);
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

    fn update(&mut self, engine: &mut ConsoleGameEngine<Self>, _dt: f32) -> bool {
        engine.clear(FG_BLACK);

        self.handle_input(engine);
        self.update_simulation();
        self.draw(engine);

        thread::sleep(Duration::from_millis(16));

        true
    }
}

fn main() {
    let mut engine = ConsoleGameEngine::new(SandSim::new());

    engine
        .construct_console(WIDTH as i16, HEIGHT as i16, 6, 6)
        .unwrap();

    engine.start();
}
