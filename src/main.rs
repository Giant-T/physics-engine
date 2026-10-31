mod app;
mod input_state;
mod physics;
mod renderer;
mod scenes;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    app::run()
}
