use requin::{play_game_ai, play_game_pvp, run_uci};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "requin",
    about = "A CLI chess program with a chess engine.",
    version = "1.3.0"
)]
struct Opt {
    #[structopt(
        short,
        long,
        default_value = "ai",
        help = "Available game modes: 'ai', 'pvp' and 'uci'"
    )]
    mode: String,
    #[structopt(short, long, default_value = "5")]
    depth: u32,
    #[structopt(
        long,
        default_value = "4",
        help = "Number of threads to use during move searches"
    )]
    num_threads: usize,
}

fn main() {
    let opt = Opt::from_args();

    match opt.mode.as_str() {
        "ai" => play_game_ai(false, opt.depth, opt.num_threads),
        "pvp" => play_game_pvp(),
        "uci" => run_uci(),
        _ => panic!("Invalid game mode."),
    }
}
