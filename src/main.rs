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
    env::args,
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
    let (_size_x, size_y) = terminal::size()?;
    match code {
        KeyCode::Left => {
            if x > 0 {
                execute!(io::stdout(), MoveTo(x - 1, y))?;
                context.last_x = x - 1;
            }
        }
        KeyCode::Right => {
            if buffer[context.top + y as usize].len() > x as usize {
                execute!(io::stdout(), MoveTo(x + 1, y))?;
                context.last_x = x + 1;
            }
        }
        KeyCode::Up => {
            let buffer_y = context.top + y as usize;
            if y > 0 {
                if (context.last_x as usize) > buffer[buffer_y - 1].len() {
                    execute!(
                        io::stdout(),
                        MoveTo(buffer[buffer_y - 1].len() as u16, y - 1)
                    )?;
                } else {
                    execute!(io::stdout(), MoveTo(context.last_x, y - 1))?;
                }
            } else if context.top > 0 {
                context.top -= 1;
                context.write_buf_to_screen(context.top)?;
                execute!(io::stdout(), MoveTo(x,y))?;
            }
        }
        KeyCode::Down => {
            let buffer_y = y as usize + context.top;

            if buffer_y + 1 < buffer.len() {
                if y != size_y-1 {
                    if (context.last_x as usize) > buffer[buffer_y + 1].len() {
                        execute!(
                            io::stdout(),
                            MoveTo(buffer[buffer_y + 1].len() as u16, y + 1)
                        )?;
                    } else {
                        execute!(io::stdout(), MoveTo(context.last_x, y + 1))?;
                    }
                } else {
                    context.top += 1;
                    execute!(io::stdout(), MoveTo(0, 0))?;
                    context.write_buf_to_screen(context.top)?;
                    if (context.last_x as usize) > buffer[buffer_y + 1].len() {
                        execute!(
                            io::stdout(),
                            MoveTo(buffer[buffer_y].len() as u16, y)
                        )?;
                    } else {
                        execute!(io::stdout(), MoveTo(context.last_x, y))?;
                    }
                }    
            }
        }
        _ => (),
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    
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

    let mut context = BufferContext {
        buffer: vec![vec![]],
        last_x: 0,
        top: 0,
        mode: BufferMode::Insert,
        path,
    };

    context.read_file(None)?;
    context.write_buf_to_screen(0)?;

    execute!(io::stdout(), MoveTo(0, 0))?;
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
                    context.write_buf_to_file()?;
                }
                Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    let (x,y) = cursor::position()?;
                    let buffer_y = y as usize + context.top;
                    context.buffer.insert(buffer_y+1, vec![]);
                    context.write_buf_to_screen(context.top)?;
                    execute!(io::stdout(), MoveTo(x, y))?;
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
