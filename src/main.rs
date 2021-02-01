use crossterm::tty::IsTty;
use liveterm::TermPrinter;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{stdin, stdout, BufRead};
use std::time::{Duration, Instant};
use structopt::StructOpt;

/// We limit to this many terminal updates per second
const FPS: u64 = 20;

#[derive(StructOpt, Copy, Clone)]
struct Opts {
    /// Reverse the sort order
    #[structopt(long, short)]
    reverse: bool,
    /// Sort in order of number of occurances
    #[structopt(long, short)]
    frequency: bool,
    /// Print each unique line once
    #[structopt(long, short, group = "format")]
    uniq: bool,
    /// Print each unique line once and include the number of occurances
    #[structopt(long, short, group = "format")]
    count: bool,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let mut vals = BTreeMap::<String, u64>::new();

    let out = stdout();
    let is_tty = out.is_tty();
    let mut tp = TermPrinter::new(out.lock());

    let mut last_print_time = Instant::now();
    for line in stdin().lock().lines() {
        *vals.entry(line.unwrap()).or_default() += 1;
        if last_print_time.elapsed() > Duration::from_millis(1000 / FPS) && is_tty {
            tp.clear()?;
            fmt_vals(opts, &vals, &mut tp.buf)?;
            tp.print()?;
            last_print_time = Instant::now();
        }
    }
    tp.clear()?;
    fmt_vals(opts, &vals, &mut tp.buf)?;
    tp.print_all()?;
    Ok(())
}

fn fmt_vals(opts: Opts, vals: &BTreeMap<String, u64>, buf: &mut String) -> Result<()> {
    use std::fmt::Write;
    buf.clear();
    // We could prevent this from allocating, but it's not worth it
    let iter = if opts.frequency {
        let mut x = vals.iter().collect::<Vec<_>>();
        x.sort_by_key(|(_, n)| *n);
        Box::new(x.into_iter()) as Box<dyn DoubleEndedIterator<Item = (&String, &u64)>>
    } else {
        Box::new(vals.iter()) as Box<dyn DoubleEndedIterator<Item = (&String, &u64)>>
    };
    let iter = if opts.reverse {
        Box::new(iter.rev()) as Box<dyn Iterator<Item = (&String, &u64)>>
    } else {
        Box::new(iter) as Box<dyn Iterator<Item = (&String, &u64)>>
    };
    for (val, n) in iter {
        if opts.count {
            writeln!(buf, "{:>7} {}", n, val)?;
        } else if opts.uniq {
            writeln!(buf, "{}", val)?;
        } else {
            for _ in 0..*n {
                writeln!(buf, "{}", val)?;
            }
        }
    }
    Ok(())
}
