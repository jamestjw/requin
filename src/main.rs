use requin::{play_game_ai, play_game_pvp};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "requin", about = "A CLI chess program with a chess engine.")]
struct Opt {
    #[structopt(short, long, default_value = "ai")]
    mode: String,
    #[structopt(short, long, default_value = "1")]
    depth: u32,
}

fn main() {
    let opt = Opt::from_args();

    match opt.mode.as_str() {
        "ai" => play_game_ai(false, opt.depth),
        "pvp" => play_game_pvp(),
        _ => panic!("Invalid game mode."),
    }
}
