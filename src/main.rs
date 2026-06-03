mod app;
mod domain;
mod infra;
mod ui;

fn main() -> std::io::Result<()> {
    app::run::run()
}
