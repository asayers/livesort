use crossterm::cursor::*;
use crossterm::style::*;
use crossterm::terminal::*;
use crossterm::*;
use std::io::{BufRead, BufReader, Write};
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
    let mut stdout = stdout.lock();
    let mut buf = vec![];
    for line in stdin.lines() {
        let line = line?;
        stdout
            .queue(MoveToPreviousLine(buf.len() as u16))?
            .queue(Clear(ClearType::FromCursorDown))?;
        buf.push(line);
        buf.sort();
        for l in &buf {
            stdout.queue(Print(l))?.queue(Print("\n"))?;
        }
        stdout.flush()?;
    }
    Ok(())
}
