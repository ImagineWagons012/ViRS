use crossterm::{
    cursor::{self, EnableBlinking, MoveDown, MoveTo, MoveToColumn, RestorePosition, SavePosition},
    event::{self, Event::Key, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    // env::args,
    io,
};

mod buffer;

use buffer::{BufferContext, BufferMode};

fn movement(
    code: KeyCode,
    _modifiers: KeyModifiers,
    context: &mut BufferContext,
) -> io::Result<()> {
    let buffer = &context.buffer;
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
        buffer: vec![vec![]],
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

    // let mut args = args();
    // let path = args.nth(1).unwrap();

    context.read_file("./src/buffer.rs".to_string())?;
    context.write_buf_to_screen(0)?;

    execute!(
        io::stdout(),
        MoveTo(
            context.buffer[context.buffer.len() - 1].len() as u16,
            context.buffer.len() as u16 - 1
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
                    context.delete_char()?;
                }
                Key(KeyEvent {
                    code: KeyCode::Delete,
                    ..
                }) => {
                    context.write_buf_to_file("test.txt".to_string())?;
                }
                Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    context.buffer.push(vec![]);
                    execute!(io::stdout(), MoveDown(1), MoveToColumn(0))?;
                }
                Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    context.write_char(c)?;
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
