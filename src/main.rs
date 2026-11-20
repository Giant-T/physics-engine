#![allow(dead_code)]
mod app;
mod input_state;
mod physics;
mod ray;
mod renderer;
mod scenes;
mod transform;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    app::run()
}
