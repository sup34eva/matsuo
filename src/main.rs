#![feature(slice_patterns, inclusive_range_syntax)]

extern crate gl;
extern crate glutin;
extern crate libc;

extern crate rand;
extern crate rayon;
extern crate docopt;

extern crate serde;
extern crate bincode;
#[macro_use]
extern crate serde_derive;

mod render;
mod display;
mod game;
mod play;
mod tree;

use docopt::Docopt;
use play::*;

static USAGE: &'static str = "
Square game with neuroevolution.

Usage:
  matsuo game
  matsuo autoplay
  matsuo (-h | --help)
  matsuo --version

Options:
  -h --help       Show this screen.
  --version       Show version.
  --size=<cells>  Size of the board in cells [default: 10].
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_game: bool,
    cmd_autoplay: bool,
    flag_size: usize,
}

fn main() {
    let args: Args = {
        Docopt::new(USAGE)
            .and_then(|d| d.deserialize())
            .unwrap_or_else(|e| e.exit())
    };

    let mode = if args.cmd_game {
        GameMode::Game
    } else if args.cmd_autoplay {
        GameMode::Autoplay
    } else {
        panic!("Invalid command")
    };

    play(args.flag_size, mode);
}
