use rand::RngExt;
use rand::rngs::SmallRng;
use rusty_console_game_engine::{color::*, key::*, prelude::*};

const WIDTH: usize = 300;
const HEIGHT: usize = 200;
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
    brush_size: i32,
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
            brush_size: 3,
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
        let terminal_velocity = 8.0;

        for y in (0..HEIGHT).rev() {
            let mut xs: Vec<usize> = (0..WIDTH).collect();
            if !left_to_right {
                xs.reverse();
            }

            for x in xs {
                let idx = Self::get_idx(x, y);
                let cell = self.grid[idx];
                if !cell.filled {
                    continue;
                }

                let mut new_vel = (cell.vel + GRAVITY).min(terminal_velocity);
                let mut cur_x = x as i32;
                let mut cur_y = y as i32;
                let move_dist = new_vel.floor() as i32;
                let mut moved = false;

                for _ in 0..move_dist.max(1) {
                    let next_y = cur_y + 1;
                    if next_y >= HEIGHT as i32 {
                        new_vel = 0.0;
                        break;
                    }

                    let down_idx = Self::get_idx(cur_x as usize, next_y as usize);
                    if !self.next_grid[down_idx].filled {
                        cur_y = next_y;
                        moved = true;
                    } else {
                        let dir = if self.rng.random_bool(0.5) { 1 } else { -1 };
                        let mut found_diagonal = false;

                        for &dx in &[dir, -dir] {
                            let nx = cur_x + dx;
                            if nx >= 0 && nx < WIDTH as i32 {
                                let diag_idx = Self::get_idx(nx as usize, next_y as usize);
                                if !self.next_grid[diag_idx].filled {
                                    cur_x = nx;
                                    cur_y = next_y;
                                    moved = true;
                                    found_diagonal = true;
                                    break;
                                }
                            }
                        }

                        if !found_diagonal {
                            new_vel = 0.0;
                            break;
                        }
                    }
                }

                let final_idx = Self::get_idx(cur_x as usize, cur_y as usize);
                self.next_grid[final_idx] = Cell {
                    filled: true,
                    color: cell.color,
                    vel: if moved { new_vel } else { 0.0 },
                };
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

        if engine.key_pressed(C) {
            self.grid.fill(Cell::default());
        }

        if engine.key_pressed(ARROW_UP) {
            self.brush_size = (self.brush_size + 1).min(20);
        }
        if engine.key_pressed(ARROW_DOWN) {
            self.brush_size = (self.brush_size - 1).max(1);
        }

        let mx = engine.mouse_x();
        let my = engine.mouse_y();

        if engine.mouse_held(LEFT) {
            for dy in -self.brush_size..=self.brush_size {
                for dx in -self.brush_size..=self.brush_size {
                    if dx * dx + dy * dy > self.brush_size * self.brush_size {
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

        engine.draw_circle_with(mx, my, self.brush_size + 1, SOLID, FG_DARK_GREY);
        engine.draw_circle_with(mx, my, self.brush_size, SOLID, color);
    }

    fn draw_pixel_char(
        &self,
        engine: &mut ConsoleGameEngine<Self>,
        x: i32,
        y: i32,
        ch: char,
        col: u16,
    ) {
        let mask: [u16; 5] = match ch.to_ascii_uppercase() {
            'A' => [0x6, 0x9, 0xF, 0x9, 0x9],
            'B' => [0xE, 0x9, 0xE, 0x9, 0xE],
            'C' => [0x7, 0x8, 0x8, 0x8, 0x7],
            'D' => [0xE, 0x9, 0x9, 0x9, 0xE],
            'E' => [0xF, 0x8, 0xE, 0x8, 0xF],
            'F' => [0xF, 0x8, 0xE, 0x8, 0x8],
            'G' => [0x7, 0x8, 0xB, 0x9, 0x7],
            'H' => [0x9, 0x9, 0xF, 0x9, 0x9],
            'I' => [0xE, 0x4, 0x4, 0x4, 0xE],
            'J' => [0x3, 0x2, 0x2, 0xA, 0x4],
            'K' => [0x9, 0xA, 0xC, 0xA, 0x9],
            'L' => [0x8, 0x8, 0x8, 0x8, 0xF],
            'M' => [0x9, 0xF, 0x9, 0x9, 0x9],
            'N' => [0x9, 0xD, 0xB, 0x9, 0x9],
            'O' => [0x6, 0x9, 0x9, 0x9, 0x6],
            'P' => [0xE, 0x9, 0xE, 0x8, 0x8],
            'Q' => [0x6, 0x9, 0x9, 0xA, 0x5],
            'R' => [0xE, 0x9, 0xE, 0xA, 0x9],
            'S' => [0x7, 0x8, 0x6, 0x1, 0xE],
            'T' => [0xF, 0x4, 0x4, 0x4, 0x4],
            'U' => [0x9, 0x9, 0x9, 0x9, 0x6],
            'V' => [0x9, 0x9, 0x9, 0xA, 0x4],
            'W' => [0x9, 0x9, 0xF, 0xF, 0x9],
            'X' => [0x9, 0x9, 0x6, 0x9, 0x9],
            'Y' => [0x9, 0x9, 0x6, 0x2, 0x2],
            'Z' => [0xF, 0x2, 0x4, 0x8, 0xF],
            '1' => [0x4, 0xC, 0x4, 0x4, 0xE],
            '2' => [0xE, 0x1, 0xE, 0x8, 0xF],
            '3' => [0xF, 0x1, 0x7, 0x1, 0xF],
            '4' => [0x9, 0x9, 0xF, 0x1, 0x1],
            '5' => [0xF, 0x8, 0xF, 0x1, 0xF],
            '6' => [0xF, 0x8, 0xF, 0x9, 0xF],
            '7' => [0xF, 0x1, 0x2, 0x4, 0x4],
            '8' => [0x6, 0x9, 0x6, 0x9, 0x6],
            '9' => [0xF, 0x9, 0xF, 0x1, 0xF],
            '0' => [0x6, 0x9, 0x9, 0x9, 0x6],
            '-' => [0x0, 0x0, 0xF, 0x0, 0x0],
            ':' => [0x0, 0x6, 0x0, 0x6, 0x0],
            '/' => [0x1, 0x2, 0x4, 0x4, 0x8],
            ' ' => [0x0, 0x0, 0x0, 0x0, 0x0],
            _ => [0xF, 0xF, 0xF, 0xF, 0xF],
        };

        for (row, &mask_row) in mask.iter().enumerate().take(5) {
            for col_idx in 0..4 {
                if (mask_row >> (3 - col_idx)) & 1 == 1 {
                    engine.draw_with(x + col_idx, y + (row as i32), SOLID, col);
                }
            }
        }
    }

    fn draw_pixel_string(
        &self,
        engine: &mut ConsoleGameEngine<Self>,
        x: i32,
        y: i32,
        text: &str,
        col: u16,
    ) {
        let mut curr_x = x;
        for ch in text.chars() {
            self.draw_pixel_char(engine, curr_x, y, ch, col);
            curr_x += 5;
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

        let mode_str = match self.tool {
            Tool::Place => "PLACE SAND",
            Tool::Delete => "DELETE",
        };

        self.draw_pixel_string(engine, 5, 5, &format!("MODE: {}", mode_str), FG_WHITE);
        self.draw_pixel_string(
            engine,
            5,
            15,
            &format!("BRUSH SIZE: {}", self.brush_size),
            FG_WHITE,
        );

        self.draw_pixel_string(engine, 5, 30, "1 - PLACE SAND", FG_GREY);
        self.draw_pixel_string(engine, 5, 40, "2 - DELETE", FG_GREY);
        self.draw_pixel_string(engine, 5, 50, "C - CLEAR ALL", FG_GREY);
        self.draw_pixel_string(engine, 5, 60, "UP/DOWN - BRUSH SIZE", FG_GREY);

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
