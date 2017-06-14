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
mod nnet;
mod evol;
mod game;
mod data;
mod play;

use docopt::Docopt;

static USAGE: &'static str = "
Square game with neuroevolution.

Usage:
  matsuo evolve
  matsuo play
  matsuo (-h | --help)
  matsuo --version

Options:
  -h --help       Show this screen.
  --version       Show version.
  --size=<cells>  Size of the board in cells [default: 10].
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_evolve: bool,
    cmd_play: bool,
    flag_size: usize,
}

fn main() {
    let args: Args = {
        Docopt::new(USAGE)
            .and_then(|d| d.deserialize())
            .unwrap_or_else(|e| e.exit())
    };

    if args.cmd_evolve {
        evol::evolve(args.flag_size);
    } else if args.cmd_play {
        play::play(args.flag_size);
    } else {
        panic!("Invalid command")
    }
}
