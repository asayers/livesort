use bpaf::Bpaf;
use crossterm::tty::IsTty;
use liveterm::TermPrinter;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{stdin, stdout, BufRead};
use std::time::{Duration, Instant};

/// We limit to this many terminal updates per second
const FPS: u64 = 20;

#[derive(Bpaf)]
#[bpaf(options)]
struct Opts {
    /// Reverse the sort order
    #[bpaf(long, short)]
    reverse: bool,
    /// Sort in order of number of occurances
    #[bpaf(long, short)]
    frequency: bool,
    #[bpaf(external, optional)]
    format: Option<Format>,
}

#[derive(Bpaf, Clone)]
enum Format {
    /// Print each unique line once
    #[bpaf(long, short)]
    Uniq,
    /// Print each unique line once and include the number of occurances
    #[bpaf(long, short)]
    Count,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let opts = opts().run();
    let mut vals = BTreeMap::<String, u64>::new();

    let out = stdout();
    let is_tty = out.is_tty();
    let mut tp = TermPrinter::new(out.lock());

    let mut last_print_time = Instant::now();
    for line in stdin().lock().lines() {
        *vals.entry(line.unwrap()).or_default() += 1;
        if last_print_time.elapsed() > Duration::from_millis(1000 / FPS) && is_tty {
            tp.clear()?;
            fmt_vals(&opts, &vals, &mut tp.buf)?;
            tp.print()?;
            last_print_time = Instant::now();
        }
    }
    tp.clear()?;
    fmt_vals(&opts, &vals, &mut tp.buf)?;
    tp.print_all()?;
    Ok(())
}

fn fmt_vals(opts: &Opts, vals: &BTreeMap<String, u64>, buf: &mut String) -> Result<()> {
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
        match opts.format {
            None => {
                for _ in 0..*n {
                    writeln!(buf, "{}", val)?;
                }
            }
            Some(Format::Uniq) => writeln!(buf, "{}", val)?,
            Some(Format::Count) => writeln!(buf, "{:>7} {}", n, val)?,
        }
    }
    Ok(())
}
