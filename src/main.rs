use crossterm::{
    cursor::{self, EnableBlinking, MoveDown, MoveTo, MoveToColumn, RestorePosition, SavePosition},
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, Write};

struct Buffer {
    buf: Vec<Vec<u8>>,
}

fn write_char(buffer: &mut Vec<Vec<u8>>, c: char) -> io::Result<()> {
    let (x, y) = cursor::position()?;
    let (size_x, _size_y) = terminal::size()?;
    if x < size_x - 1 {
        execute!(io::stdout(), MoveTo(x + 1, y))?;
        buffer[y as usize].push(c as u8);
    } else {
        execute!(io::stdout(), MoveTo(1, y + 1))?;
        buffer.push(vec![]);
        buffer[y as usize].push(c as u8);
    }
    Ok(())
}

fn delete_char(buffer: &mut Vec<Vec<u8>>) -> io::Result<()> {
    let (x, y) = cursor::position()?;
    if x != 0 {
        buffer[y as usize].remove((x - 1) as usize);
        execute!(io::stdout(), MoveTo(x - 1, y))?;
    }
    if buffer.len() != 1 && buffer[y as usize].len() < 1 {
        buffer.remove(y as usize);
        execute!(io::stdout(), MoveTo(x, y - 1))?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut buffer = Buffer { buf: vec![vec![]] };
    execute!(
        std::io::stdout(),
        SavePosition,
        EnterAlternateScreen,
        EnableBlinking,
        Clear(ClearType::All),
        MoveTo(0, 0),
        DisableLineWrap,
    )?;
    terminal::enable_raw_mode()?;

    execute!(io::stdout(), MoveTo(0, 0))?;
    for (i, line) in buffer.buf.iter().enumerate() {
        execute!(io::stdout(), MoveTo(0, i as u16))?;
        io::stdout().write_all(line)?;
    }
    execute!(
        io::stdout(),
        MoveTo(
            buffer.buf[buffer.buf.len() - 1].len() as u16,
            buffer.buf.len() as u16 - 1
        )
    )?;
    'a: loop {
        if let Ok(event) = event::read() {
            match event {
                event::Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    break 'a;
                }
                event::Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    delete_char(&mut buffer.buf)?;
                    let (x, y) = cursor::position()?;
                    execute!(io::stdout(), MoveTo(x, y))?;
                    io::stdout().write_all(
                        vec![' ' as u8; buffer.buf[y as usize][x as usize..buffer.buf[y as usize].len()].len() + 1].as_ref(),
                    )?;
                    execute!(io::stdout(), MoveTo(0, y))?;
                    io::stdout().write_all(&buffer.buf[y as usize][x as usize..buffer.buf[y as usize].len()])?;
                    execute!(io::stdout(), MoveTo(x, y))?;
                }
                event::Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    buffer.buf.push(vec![]);
                    execute!(io::stdout(), MoveDown(1), MoveToColumn(0))?;
                }
                event::Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    write_char(&mut buffer.buf, c)?;
                    let (x, y) = cursor::position()?;
                    execute!(io::stdout(), MoveTo(0, y))?;
                    io::stdout()
                        .write_all(vec![0; terminal::size().unwrap().1 as usize].as_ref())?;
                    io::stdout().write_all(buffer.buf[y as usize].as_ref())?;
                    execute!(io::stdout(), MoveTo(x, y))?;
                }
                _ => (),
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(
        std::io::stdout(),
        LeaveAlternateScreen,
        RestorePosition,
        EnableLineWrap,
    )?;
    Ok(())
}
