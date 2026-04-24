use rusty_console_game_engine::prelude::*;

struct SandSim;

impl ConsoleGame for SandSim {
    fn app_name(&self) -> &str {
        "Falling Sand Simulation"
    }

    fn create(&mut self, _engine: &mut ConsoleGameEngine<Self>) -> bool {
        true
    }

    fn update(&mut self, engine: &mut ConsoleGameEngine<Self>, _elapsed_time: f32) -> bool {
        engine.clear(FG_BLACK);

        true
    }
}

fn main() {
    let mut engine = ConsoleGameEngine::new(SandSim);
    engine
        .construct_console(150, 150, 4, 4)
        .expect("Console Construction Failed");
    engine.start();
}
