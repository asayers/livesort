use crossterm::{cursor, terminal, terminal::ClearType, QueueableCommand};
use std::collections::BTreeSet;
use std::error::Error;
use std::io::{stdin, stdout, BufRead, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let mut vals = BTreeSet::new();
    let mut last_print = 0;
    let out = stdout();
    let mut out = out.lock();
    for line in stdin().lock().lines() {
        vals.insert(line.unwrap());
        out.queue(cursor::MoveToPreviousLine(last_print))?
            .queue(terminal::Clear(ClearType::FromCursorDown))?;
        let (_, height) = terminal::size()?;
        last_print = (height - 1).min(vals.len() as u16);
        for val in vals.iter().take(last_print as usize) {
            out.write_all(val.as_bytes())?;
            out.write_all(b"\n")?;
        }
        out.flush()?;
    }
    Ok(())
}
