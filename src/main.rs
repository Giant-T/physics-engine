mod app;
mod renderer;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    app::run()
}
