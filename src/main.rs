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
    #[structopt(long, short)]
    count: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let mut vals = BTreeMap::<String, u64>::new();

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
            let len = if opts.count || opts.uniq {
                vals.len()
            } else {
                vals.values().map(|&x| x as usize).sum()
            };
            let n = (height as usize - 1).min(len);
            print_vals(opts, &vals, len - n, &mut out)?;
            out.flush()?;
            last_print_rows = u16::try_from(n).unwrap();
            last_print_time = Instant::now();
        }
    }
    if last_print_rows != 0 {
        out.queue(cursor::MoveToPreviousLine(last_print_rows))?
            .queue(terminal::Clear(ClearType::FromCursorDown))?;
    }
    print_vals(opts, &vals, 0, &mut out)?;
    out.flush()?;
    Ok(())
}

fn print_vals(
    opts: Opts,
    vals: &BTreeMap<String, u64>,
    skip: usize,
    mut out: impl Write,
) -> Result<(), Box<dyn Error>> {
    // We could prevent this from allocating, but it's not worth it
    let iter = if opts.reverse {
        Box::new(vals.iter().rev()) as Box<dyn Iterator<Item = (&String, &u64)>>
    } else {
        Box::new(vals.iter()) as Box<dyn Iterator<Item = (&String, &u64)>>
    };
    if opts.count {
        for (val, n) in iter.skip(skip) {
            writeln!(out, "{:>7} {}", n, val)?;
        }
    } else if opts.uniq {
        for val in iter.map(|(s, _)| s).skip(skip) {
            writeln!(out, "{}", val)?;
        }
    } else {
        for val in iter
            .flat_map(|(s, n): (&String, &u64)| std::iter::repeat(s).take(*n as usize))
            .skip(skip)
        {
            writeln!(out, "{}", val)?;
        }
    }
    Ok(())
}
