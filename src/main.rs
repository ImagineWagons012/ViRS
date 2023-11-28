use crossterm::{
    cursor::{self, EnableBlinking, MoveDown, MoveTo, MoveToColumn, RestorePosition, SavePosition},
    event::{self, Event::Key, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{io::{self, Write}, env::args};

struct BufferContext {
    buf: Vec<Vec<char>>,
    last_x: u16,
    mode: BufferMode,
}

impl BufferContext {
    fn write_buf_to_screen(&self) -> io::Result<()> {
        execute!(io::stdout(), MoveTo(0, 0))?;
        for (i, line) in self.buf.iter().enumerate() {
            execute!(io::stdout(), MoveTo(0, i as u16))?;
            let mut final_line = vec![];
            let mut buf = [0; 4];
            for char in line {
                char.encode_utf8(&mut buf);
                for byte in buf {
                    final_line.push(byte);
                }
            }
            io::stdout().write_all(&final_line)?;
            execute!(io::stdout(), MoveDown(1))?;
        }
        Ok(())
    }
    fn read_file(&mut self, path: String) -> io::Result<()> {
        Ok(())
    }
}

enum BufferMode {
    Normal,
    Insert,
}

fn overwrite_char(pos: (u16, u16), context: &mut BufferContext, c: char) -> io::Result<()> {
    let (x, y) = pos;
    let buffer = &mut context.buf;

    let deletion_length = buffer[y as usize][x as usize..buffer[y as usize].len()]
        .iter()
        .map(|x| x.len_utf8())
        .sum::<usize>();

    let char_buf = &mut [0; 4];
    c.encode_utf8(char_buf);

    io::stdout().write_all(vec![0; deletion_length].as_ref())?;
    execute!(io::stdout(), MoveTo(x, y))?;
    io::stdout().write_all(&char_buf[0..c.len_utf8()])?;

    Ok(())
}

fn write_char(context: &mut BufferContext, c: char) -> io::Result<()> {
    let buffer = &mut context.buf;
    let (x, y) = cursor::position()?;
    let (size_x, _size_y) = terminal::size()?;
    if x < size_x - 1 {
        buffer[y as usize].insert(x as usize, c);
    } else {
        execute!(io::stdout(), MoveTo(0, y + 1))?;
        buffer.push(vec![]);
        buffer[(y + 1) as usize].push(c);
    }

    let (x, y) = cursor::position()?;
    overwrite_char((x, y), context, c)?;

    execute!(io::stdout(), MoveTo(x + 1, y))?;
    context.last_x = cursor::position()?.0;

    Ok(())
}

fn delete_char(context: &mut BufferContext) -> io::Result<()> {
    let buffer = &mut context.buf;

    let (x, y) = cursor::position()?;
    let mut deletion_length = 0;
    if x > 0 {
        deletion_length = buffer[y as usize][(x - 1) as usize..buffer[y as usize].len()]
            .iter()
            .map(|x| x.len_utf8())
            .sum::<usize>();
        buffer[y as usize].remove((x - 1) as usize);
        execute!(io::stdout(), MoveTo(x - 1, y))?;
    } else {
        if buffer.len() != 1 && buffer[y as usize].len() < 1 {
            buffer.remove(y as usize);
            execute!(
                io::stdout(),
                MoveTo(buffer[y as usize - 1].len() as u16, y - 1)
            )?;
        }
    }

    let (x, y) = cursor::position()?;

    io::stdout().write_all(vec![b' '; deletion_length].as_ref())?;

    execute!(io::stdout(), MoveTo(x, y))?;

    let mut final_printout = vec![];
    for char in buffer[y as usize][x as usize..buffer[y as usize].len()].iter() {
        let char_buf = &mut [0; 4];
        char.encode_utf8(char_buf);
        for byte in char_buf {
            final_printout.push(*byte);
        }
    }
    io::stdout().write_all(&final_printout.as_ref())?;

    execute!(io::stdout(), MoveTo(x, y))?;

    let (x, _) = cursor::position()?;
    context.last_x = x;

    Ok(())
}

fn movement(
    code: KeyCode,
    _modifiers: KeyModifiers,
    context: &mut BufferContext,
) -> io::Result<()> {
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
                    execute!(
                        io::stdout(),
                        MoveTo(buffer[y as usize - 1].len() as u16, y - 1)
                    )?;
                } else {
                    execute!(io::stdout(), MoveTo(context.last_x, y - 1))?;
                }
            }
        }
        KeyCode::Down => {
            if (y as usize) < buffer.len() - 1 {
                if (context.last_x as usize) > buffer[y as usize + 1].len() {
                    execute!(
                        io::stdout(),
                        MoveTo(buffer[y as usize + 1].len() as u16, y + 1)
                    )?;
                } else {
                    execute!(io::stdout(), MoveTo(context.last_x, y + 1))?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut context = BufferContext {
        buf: vec![vec![]],
        last_x: 0,
        mode: BufferMode::Insert,
    };
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

    let mut args = args();
    let path = args.nth(1).unwrap();

    context.write_buf_to_screen()?;

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
