use crossterm::{cursor, terminal, terminal::ClearType, QueueableCommand};
use std::error::Error;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Print to a terminal, clearing the stuff that was written last time.
///
/// Here's how to use it:
///
/// ```
/// let mut tp = TermPrinter::new(std::io::stdout());
/// for i in 0..10 {
///     tp.clear()?;              // clear what we draw last time
///     tp.buf.clear();           // clear the buffer
///     write!(tp.buf, "{}", i)?; // fill the buffer
///     tp.print()?;              // draw the buffer
/// }
/// ```
pub struct TermPrinter<W> {
    wtr: W,
    pub buf: String,
    /// Last time, we started at this byte in `buf` and printed to the end.
    last_print_start: usize,
}

impl<W: Write> TermPrinter<W> {
    pub fn new(wtr: W) -> TermPrinter<W> {
        TermPrinter {
            wtr,
            buf: String::new(),
            last_print_start: 0,
        }
    }

    /// Clear the lines we wrote previously.
    ///
    /// Note that in some cases this is impossible: for instance, if we print N
    /// lines and then the user resizes the terminal to something less than N,
    /// those lines _will_ end up in the scrollback buffer - there's nothing
    /// we can do about that (without switching to the alternate screen).
    pub fn clear(&mut self) -> Result<()> {
        // Looks like MoveToPreviousLine(0) still moves up one line, so we
        // need to guard the 0 case
        if self.last_print_start != self.buf.len() {
            let (width, _) = terminal::size()?;
            let line_starts = soft_breaks(&self.buf[self.last_print_start..], width as usize);
            let n = line_starts.len() as u16;
            self.wtr
                .queue(cursor::MoveToPreviousLine(n))?
                .queue(terminal::Clear(ClearType::FromCursorDown))?
                .flush()?;
            self.last_print_start = self.buf.len();
        }
        Ok(())
    }

    /// Print the visible tail of `buf` to `wtr`.
    ///
    /// Stuff that scrolls off the top of the terminal end up in the scrollback
    /// buffer, where we can't clear it.  Therefore this method will only
    /// write as many lines as the terminal currently has room for.
    pub fn print(&mut self) -> Result<()> {
        let (width, height) = terminal::size()?;
        let line_starts = soft_breaks(&self.buf, width as usize);
        let len = line_starts.len();
        let n = (height as usize - 1).min(len);
        let start = line_starts[len - n];
        self.wtr.write_all(&self.buf.as_bytes()[start..])?;
        self.wtr.flush()?;
        self.last_print_start = start;
        Ok(())
    }

    /// Print all of `buf` to `wtr`.
    ///
    /// After this we can't reliably clear what we've written (since it
    /// may have gone off the top of the screen).  Hence, this method drops
    /// the `TermPrinter`.
    pub fn print_all(mut self) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
