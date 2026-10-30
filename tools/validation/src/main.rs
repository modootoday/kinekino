mod application;
mod domain;
mod infrastructure;

fn main() -> anyhow::Result<()> {
    application::run(std::env::args().skip(1).collect())
}
