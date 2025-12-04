use clap::Parser;
use example::{process, Opt};

fn main() {
    let opt = Opt::parse();
    process(&opt);
}
