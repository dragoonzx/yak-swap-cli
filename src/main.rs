use terminal::Terminal;

mod abis;
mod db;
mod network;
mod query;
mod settings;
mod swap;
mod terminal;
mod token;
mod wallet;

fn main() {
    Terminal::render_on_launch();
}
