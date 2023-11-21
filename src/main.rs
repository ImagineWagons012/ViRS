use crossterm::{
    cursor::{self, EnableBlinking, MoveTo, MoveDown, MoveToColumn, RestorePosition, SavePosition},
    event::{self, Event::Key, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, Write};

struct BufferContext {
    buf: Vec<Vec<u8>>,
    last_x: u16,
    mode: BufferMode,
}

enum BufferMode {
    Normal,
    Insert,
}

fn write_char(context: &mut BufferContext, c: char) -> io::Result<()> {
    let buffer = &mut context.buf;
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
    
    
    let (x, y) = cursor::position()?;
    execute!(io::stdout(), MoveTo(0, y))?;
    io::stdout().write_all(vec![0; buffer[y as usize].len()].as_ref())?;
    io::stdout().write_all(buffer[y as usize].as_ref())?;
    execute!(io::stdout(), MoveTo(x, y))?;

    context.last_x = x;

    Ok(())
}

fn delete_char(context: &mut BufferContext) -> io::Result<()> {
    let buffer = &mut context.buf;

    let (x, y) = cursor::position()?;
    if x != 0 {
        buffer[y as usize].remove((x - 1) as usize);
        execute!(io::stdout(), MoveTo(x - 1, y))?;
    }

    let (x, y) = cursor::position()?;
    execute!(io::stdout(), MoveTo(x, y))?;
    io::stdout().write_all(
        vec![' ' as u8; buffer[y as usize][x as usize..buffer[y as usize].len()].len() + 1]
            .as_ref(),
    )?;
    execute!(io::stdout(), MoveTo(0, y))?;
    io::stdout().write_all(&buffer[y as usize][x as usize..buffer[y as usize].len()])?;
    execute!(io::stdout(), MoveTo(x, y))?;

    if buffer.len() != 1 && buffer[y as usize].len() < 1 {
        buffer.remove(y as usize);
        execute!(
            io::stdout(),
            MoveTo(buffer[y as usize - 1].len() as u16, y - 1)
        )?;
    }

    context.last_x = x;

    Ok(())
}

fn movement(code: KeyCode, _modifiers: KeyModifiers, context: &mut BufferContext) -> io::Result<()> {
    let buffer = &context.buf;
    let (x, y) = cursor::position()?;
    match code {
        KeyCode::Left => {
            if x > 0 {
                execute!(io::stdout(), MoveTo(x - 1, y))?;
                context.last_x = x - 1;
            }
        }
        KeyCode::Right => {
            if buffer[y as usize].len() > x as usize {
                execute!(io::stdout(), MoveTo(x + 1, y))?;
                context.last_x = x + 1;
            }
        }
        KeyCode::Up => {
            if y > 0 {
                if (context.last_x as usize) > buffer[y as usize - 1].len() {
                    execute!(io::stdout(), MoveTo(buffer[y as usize - 1].len() as u16, y - 1))?;
                }
                else {
                    execute!(io::stdout(), MoveTo(context.last_x, y - 1))?;
                }
            }
        }
        KeyCode::Down => {
            if (y as usize) < buffer.len() - 1 {
                if (context.last_x as usize) > buffer[y as usize + 1].len() {
                    execute!(io::stdout(), MoveTo(buffer[y as usize + 1].len() as u16, y + 1))?;
                }
                else {
                    execute!(io::stdout(), MoveTo(context.last_x, y + 1))?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut context = BufferContext { buf: vec![vec![]], last_x: 0, mode: BufferMode::Insert };
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
    for (i, line) in context.buf.iter().enumerate() {
        execute!(io::stdout(), MoveTo(0, i as u16))?;
        io::stdout().write_all(line)?;
    }
    execute!(
        io::stdout(),
        MoveTo(
            context.buf[context.buf.len() - 1].len() as u16,
            context.buf.len() as u16 - 1
        )
    )?;
    'a: loop {
        if let Ok(event) = event::read() {
            match event {
                Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    break 'a;
                }
                Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    delete_char(&mut context)?;
                }
                Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    context.buf.push(vec![]);
                    execute!(io::stdout(), MoveDown(1), MoveToColumn(0))?;
                }
                Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    write_char(&mut context, c)?;
                }
                Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    movement(code, modifiers, &mut context)?;
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
