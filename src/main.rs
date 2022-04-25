#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

mod app;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output stream. defaults to console://. Other output supports
    /// redis://[user:password@]<address[:port]>
    /// ws[s]://address[:port]/[prefix]
    #[clap(short, long, default_value_t = String::from("console://"))]
    output: String,

    #[clap(short, long, default_value_t = 2*1024)]
    tail: u64,
    /// enable debug logs
    #[clap(short, long)]
    debug: bool,

    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    })?;

    let mut out = output::output(&args.output)?;

    app::tail(args.file, args.tail, &mut out)
}
