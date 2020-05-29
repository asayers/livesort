use crossterm::cursor::*;
use crossterm::style::*;
use crossterm::terminal::*;
use crossterm::*;
use std::io::{BufRead, BufReader, BufWriter, Write};
fn main() {
    match main_2() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
fn main_2() -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stdin = BufReader::new(stdin.lock());
    let mut stdout = BufWriter::new(stdout.lock());
    let mut buf = vec![];
    let mut printed = 0;
    macro_rules! clear {
        () => {
            // We don't want to clear any line if we haven't printed yet
            if printed > 0 {
                stdout
                    .queue(MoveToPreviousLine(printed as u16))?
                    .queue(Clear(ClearType::FromCursorDown))?;
            }
        };
    }
    for line in stdin.lines() {
        let line = line?;
        buf.push(line);
        buf.sort();
        clear!();
        printed = buf.len().min(usize::from(size()?.1 - 1));
        for l in buf.iter().skip(buf.len() - printed) {
            stdout.queue(Print(l))?.queue(Print("\n"))?;
        }
        stdout.flush()?;
    }
    clear!();
    for l in &buf {
        stdout.queue(Print(l))?.queue(Print("\n"))?;
    }
    stdout.flush()?;
    Ok(())
}
