#[macro_use]
extern crate log;

mod output;

use anyhow::{Context, Result};
use clap::Parser;
use inotify::{EventMask, Inotify, WatchMask};
use output::{Console, Redis};
use std::fs::OpenOptions;
use std::io::{copy, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

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

fn tail<P: AsRef<Path>>(path: P, tail: u64, out: &mut dyn Write) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(path.as_ref())
        .with_context(|| format!("failed to open file {:?}", path.as_ref()))?;

    match file.metadata().context("failed to get file stat")?.size() {
        size if size > tail => file
            .seek(SeekFrom::End(-(tail as i64)))
            .context("failed to seek file")?,
        _ => 0,
    };

    copy(&mut file, out).context("failed to copy file content")?;

    let mut notify = Inotify::init()?;
    notify
        .add_watch(path, WatchMask::MODIFY)
        .context("failed to add file watch")?;

    let mut event_buffer = [0u8; 4096];

    let mut prev_size: u64 = 0;
    loop {
        let events = notify
            .read_events_blocking(&mut event_buffer)
            .context("failed to read inotify events")?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                debug!("file modified event");
                let size = file.metadata()?.size();
                if size < prev_size {
                    // file has been truncated. We need to seek
                    // to beginning of file before reading
                    file.seek(SeekFrom::Start(0))?;
                }

                prev_size = size;

                loop {
                    match copy(&mut file, out) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(err) => {
                            error!("failed to send data to output stream: {}", err);
                        }
                    }
                }
            }
        }
    }
}

fn output<S: AsRef<str>>(url: S) -> Result<Box<dyn Write>> {
    if true {
        return Ok(Box::new(Console {}));
    } else {
        return Ok(Box::new(Redis::new()?));
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else {
        log::Level::Info
    })?;

    let mut out = output(&args.output)?;

    //let mut console = Console {};
    return tail(args.file, args.tail, &mut out);
}
