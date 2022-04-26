use anyhow::{Context, Result};
use inotify::{EventMask, Inotify, WatchMask};
use std::fs::OpenOptions;
use std::io::{copy, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

pub fn tail<P: AsRef<Path>>(path: P, tail: u64, out: &mut dyn Write) -> Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(path.as_ref())
        .with_context(|| format!("cannot open file {:?}", path.as_ref()))?;

    match file.metadata().context("failed to get file stat")?.size() {
        size if size > tail => {
            file.seek(SeekFrom::End(-(tail as i64)))
                .context("failed to seek file")?;
        }
        _ => (),
    };

    copy(&mut file, out).context("failed to tail file content")?;

    let mut notify = Inotify::init()?;
    notify
        .add_watch(path, WatchMask::MODIFY)
        .context("failed to add file watch")?;

    let mut event_buffer = [0u8; 4 * 1024];
    let mut prev_size: u64 = 0;

    loop {
        let events = notify
            .read_events_blocking(&mut event_buffer)
            .context("failed to read inotify events")?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
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
