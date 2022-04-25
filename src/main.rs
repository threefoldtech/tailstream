#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

mod app;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// A tail like tool but tail the file to a configurable output module
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Output stream. defaults to console://. Other output supports
    /// redis://[user:password@]<address[:port]>
    /// ws[s]://address[:port]/[prefix]
    #[clap(short, long, default_value_t = String::from("console://"))]
    output: String,

    /// compression, compresses the log message (per chunk) so each
    /// chunk of logs can be decompressed separately from previous chunks
    /// ignored in case of `console` output
    #[clap(short, long, default_value_t = output::CompressionKind::Gzip)]
    compression: output::CompressionKind,

    #[clap(short, long, default_value_t = 2*1024)]
    tail: u64,
    /// enable debug logs
    #[clap(short, long)]
    debug: bool,

    /// file to watch
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    })?;

    let mut out = output::output(&args.output, args.compression)?;

    app::tail(args.file, args.tail, &mut out)
}
