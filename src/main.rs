use crossterm::{
    execute,
    cursor::{EnableBlinking, MoveTo, RestorePosition, SavePosition},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, KeyEvent, KeyCode},
};

fn main() -> std::io::Result<()> {
    execute!(
        std::io::stdout(),
        SavePosition,
        EnterAlternateScreen,
        EnableBlinking,
        Clear(ClearType::All),
        MoveTo(0, 0),
    )?;

    'a: loop {
        if let Ok(event) = event::read() {
            match event {
                event::Event::Key(KeyEvent {code: KeyCode::Esc, ..}) => {
                    break 'a;
                },
                event::Event::Key(KeyEvent { code: KeyCode::Backspace, ..}) => {
                    break 'a;
                },
                _ => ()
            }
        }
    }

    execute!(
        std::io::stdout(),
        LeaveAlternateScreen,
        RestorePosition,
    )?;
 Ok(())
}

