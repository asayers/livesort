use crossterm::{cursor, terminal, terminal::ClearType, QueueableCommand};
use std::collections::BTreeSet;
use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};
use std::time::{Duration, Instant};
use structopt::StructOpt;

/// We limit to this many terminal updates per second
const FPS: u64 = 20;

#[derive(StructOpt)]
struct Opts {
    #[structopt(long, short)]
    uniq: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let mut vals = BTreeSet::new();
    let mut last_print_rows = 0;
    let mut last_print_time = Instant::now();
    let out = stdout();
    let mut out = out.lock();
    out.write_all(b"\n")?; // Why? I don't know...
    for line in stdin().lock().lines() {
        vals.insert(line.unwrap());
        if last_print_time.elapsed() > Duration::from_millis(1000 / FPS) {
            out.queue(cursor::MoveToPreviousLine(last_print_rows))?
                .queue(terminal::Clear(ClearType::FromCursorDown))?;
            let (_, height) = terminal::size()?;
            last_print_rows = (height - 1).min(vals.len() as u16);
            for val in vals.iter().take(last_print_rows as usize) {
                out.write_all(val.as_bytes())?;
                out.write_all(b"\n")?;
            }
            out.flush()?;
            last_print_time = Instant::now();
        }
    }
    out.queue(cursor::MoveToPreviousLine(last_print_rows))?
        .queue(terminal::Clear(ClearType::FromCursorDown))?;
    for val in &vals {
        out.write_all(val.as_bytes())?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}
