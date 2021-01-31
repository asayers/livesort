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

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let mut vals = BTreeMap::<String, u64>::new();

    let mut last_print_time = Instant::now();
    let out = stdout();
    let mut tp = TermPrinter::new(out.lock());
    let mut buf = String::new();
    for line in stdin().lock().lines() {
        *vals.entry(line.unwrap()).or_default() += 1;
        if last_print_time.elapsed() > Duration::from_millis(1000 / FPS) && out.is_tty() {
            fmt_vals(opts, &vals, &mut buf)?;
            tp.clear()?;
            tp.print(&buf)?;
            tp.wtr.flush()?;
            last_print_time = Instant::now();
        }
    }
    fmt_vals(opts, &vals, &mut buf)?;
    tp.clear()?;
    tp.wtr.write_all(buf.as_bytes())?;
    tp.wtr.flush()?;
    Ok(())
}

fn fmt_vals(opts: Opts, vals: &BTreeMap<String, u64>, buf: &mut String) -> Result<()> {
    use std::fmt::Write;
    buf.clear();
    // We could prevent this from allocating, but it's not worth it
    let iter = if opts.reverse {
        Box::new(vals.iter().rev()) as Box<dyn Iterator<Item = (&String, &u64)>>
    } else {
        Box::new(vals.iter()) as Box<dyn Iterator<Item = (&String, &u64)>>
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

struct TermPrinter<W> {
    wtr: W,
    last_print_rows: u16,
}

impl<W: Write> TermPrinter<W> {
    fn new(wtr: W) -> TermPrinter<W> {
        TermPrinter {
            wtr,
            last_print_rows: 0,
        }
    }
    fn clear(&mut self) -> Result<()> {
        // Looks like MoveToPreviousLine(0) still moves up one line, so we
        // need to guard the 0 case
        if self.last_print_rows != 0 {
            self.wtr
                .queue(cursor::MoveToPreviousLine(self.last_print_rows))?
                .queue(terminal::Clear(ClearType::FromCursorDown))?;
            self.last_print_rows = 0;
        }
        Ok(())
    }
    fn print(&mut self, s: &str) -> Result<()> {
        let (_, height) = terminal::size()?;
        let len = s.lines().count();
        let n = (height as usize - 1).min(len);
        for line in s.lines().skip(len - n) {
            writeln!(self.wtr, "{}", line)?;
        }
        self.last_print_rows = u16::try_from(n).unwrap();
        Ok(())
    }
}

fn soft_breaks(s: &str, width: usize) -> Vec<usize> {
    let mut line_starts = vec![0];
    let mut push = |x: usize| line_starts.push(line_starts.last().unwrap() + x);
    for line in s.lines() {
        let len = line.len();
        for _ in 0..len / width {
            push(width);
        }
        push(len % width + 1);
    }
    line_starts.pop(); // Drop the last one
    line_starts
}

#[test]
fn test_soft_breaks() {
    assert_eq!(soft_breaks("foo", 100), vec![0]);
    assert_eq!(soft_breaks("foobarqux", 5), vec![0, 5]);
    assert_eq!(soft_breaks("foobarqux\n", 5), vec![0, 5]);
    assert_eq!(soft_breaks("foo\nbar\nqux", 100), vec![0, 4, 8]);
    assert_eq!(
        soft_breaks("foo\nfoobarquxzap\nfoo", 5),
        vec![0, 4, 9, 14, 17]
    );
    assert_eq!(&"foo\nfoobarquxzap\nfoo"[0..4], "foo\n");
    assert_eq!(&"foo\nfoobarquxzap\nfoo"[4..9], "fooba");
    assert_eq!(&"foo\nfoobarquxzap\nfoo"[9..14], "rquxz");
    assert_eq!(&"foo\nfoobarquxzap\nfoo"[14..17], "ap\n");
    assert_eq!(&"foo\nfoobarquxzap\nfoo"[17..], "foo");
}
