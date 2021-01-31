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

    let out = stdout();
    let mut tp = TermPrinter::new(out.lock());

    let mut last_print_time = Instant::now();
    for line in stdin().lock().lines() {
        *vals.entry(line.unwrap()).or_default() += 1;
        if last_print_time.elapsed() > Duration::from_millis(1000 / FPS) && out.is_tty() {
            tp.clear()?;
            fmt_vals(opts, &vals, &mut tp.buf)?;
            tp.print()?;
            last_print_time = Instant::now();
        }
    }
    tp.clear()?;
    fmt_vals(opts, &vals, &mut tp.buf)?;
    tp.print_unconstrained()?;
    Ok(())
}

fn fmt_vals(opts: Opts, vals: &BTreeMap<String, u64>, buf: &mut String) -> Result<()> {
    use std::fmt::Write;
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
    buf: String,
    last_print_rows: u16,
}

impl<W: Write> TermPrinter<W> {
    fn new(wtr: W) -> TermPrinter<W> {
        TermPrinter {
            wtr,
            buf: String::new(),
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
            self.buf.clear();
        }
        Ok(())
    }
    fn print(&mut self) -> Result<()> {
        let (width, height) = terminal::size()?;
        let line_starts = soft_breaks(&self.buf, width as usize);
        let len = line_starts.len();
        let n = (height as usize - 1).min(len);
        let start = line_starts[len - n];
        self.wtr.write_all(&self.buf.as_bytes()[start..])?;
        self.wtr.flush()?;
        self.last_print_rows = u16::try_from(n).unwrap();
        Ok(())
    }
    /// After this we can't reliably clear what we've written (since it
    /// may have gone off the top of the screen).  Hence, this method drops
    /// the `TermPrinter`.
    fn print_unconstrained(mut self) -> Result<()> {
        self.wtr.write_all(self.buf.as_bytes())?;
        self.wtr.flush()?;
        Ok(())
    }
}

fn soft_breaks(s: &str, width: usize) -> Vec<usize> {
    let mut line_starts = vec![0];
    let mut push = |x: usize| line_starts.push(line_starts.last().unwrap() + x);
    // FIXME: lines() will break on both \n and \r\n.  However, we assume that
    // there's only one byte between successive lines (that's the +1 below).
    // This means this function is broken for \r\n-terminated documents.
    // See test_soft_breaks_rn for an example.
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

#[test]
#[should_panic]
// FIXME: This is a bug
fn test_soft_breaks_rn() {
    assert_eq!(soft_breaks("foo\r\nbar\r\nqux", 100), vec![0, 5, 10]);
}
