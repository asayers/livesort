use crossterm::{cursor, terminal, terminal::ClearType, tty::IsTty, QueueableCommand};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};
use std::time::{Duration, Instant};
use structopt::StructOpt;

/// We limit to this many terminal updates per second
const FPS: u64 = 20;

#[derive(StructOpt, Copy, Clone)]
struct Opts {
    #[structopt(long, short)]
    uniq: bool,
    #[structopt(long, short)]
    reverse: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let mut vals = BTreeMap::<String, u16>::new();

    let mut last_print_rows = 0;
    let mut last_print_time = Instant::now();
    let out = stdout();
    let mut out = out.lock();
    for line in stdin().lock().lines() {
        *vals.entry(line.unwrap()).or_default() += 1;
        if last_print_time.elapsed() > Duration::from_millis(1000 / FPS) && out.is_tty() {
            if last_print_rows != 0 {
                // Looks like MoveToPreviousLine(0) still moves up one line,
                // so we need to guard the 0 case
                out.queue(cursor::MoveToPreviousLine(last_print_rows))?
                    .queue(terminal::Clear(ClearType::FromCursorDown))?;
            }
            let (_, height) = terminal::size()?;
            let len = if opts.uniq {
                vals.len()
            } else {
                vals.values().map(|&x| x as usize).sum()
            };
            let n = (height as usize - 1).min(len);
            for val in iter(opts, &vals).skip(len - n) {
                out.write_all(val.as_bytes())?;
                out.write_all(b"\n")?;
            }
            out.flush()?;
            last_print_rows = u16::try_from(n).unwrap();
            last_print_time = Instant::now();
        }
    }
    out.queue(cursor::MoveToPreviousLine(last_print_rows))?
        .queue(terminal::Clear(ClearType::FromCursorDown))?;
    for val in iter(opts, &vals) {
        out.write_all(val.as_bytes())?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}

fn iter(opts: Opts, vals: &BTreeMap<String, u16>) -> Box<dyn Iterator<Item = &String> + '_> {
    // We could prevent this from allocating, but it's not worth it
    let iter = if opts.reverse {
        Box::new(vals.iter().rev()) as Box<dyn Iterator<Item = (&String, &u16)>>
    } else {
        Box::new(vals.iter()) as Box<dyn Iterator<Item = (&String, &u16)>>
    };
    if opts.uniq {
        Box::new(iter.map(|(s, _)| s)) as Box<dyn Iterator<Item = &String>>
    } else {
        Box::new(iter.flat_map(|(s, n): (&String, &u16)| std::iter::repeat(s).take(*n as usize)))
            as Box<dyn Iterator<Item = &String>>
    }
}
