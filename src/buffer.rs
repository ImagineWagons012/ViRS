use crossterm::{
    cursor::{self, MoveDown, MoveTo, MoveToColumn},
    execute, terminal,
};

use std::fs::{self, read_to_string};
use std::{
    io::{self, Write},
    vec,
};

pub struct BufferContext {
    pub buffer: Vec<Vec<char>>,
    pub last_x: u16,
    pub mode: BufferMode,
}

impl BufferContext {
    pub fn write_buf_to_screen(&self, mut from_row: usize) -> io::Result<()> {
        if from_row == self.buffer.len() {
            from_row = 0;
        }
        let (_, size_y) = (terminal::size()?.0 as usize, terminal::size()?.1 as usize);
        for i in from_row..size_y {
            execute!(io::stdout(), MoveTo(0, i as u16))?;
            io::stdout().write_all(&[0;1024])?;
        }
        for (i, line) in self.buffer.iter().enumerate() {
            if i >= from_row {
                if i > size_y {
                    break;
                }
                execute!(io::stdout(), MoveTo(0, (i) as u16))?;
                let mut final_line = vec![];
                let mut buf = [0; 4];
                for char in line {
                    char.encode_utf8(&mut buf);
                    for byte in buf {
                        final_line.push(byte);
                    }
                }
                io::stdout().write_all(&final_line)?;
                execute!(io::stdout(), MoveDown(1), MoveToColumn(0))?;
            }
        }
        Ok(())
    }
    pub fn read_file(&mut self, path: String) -> io::Result<()> {
        let file_string = read_to_string(path)?;
        let lines = file_string.split('\n').collect::<Vec<&str>>();
        for (i, line) in lines.iter().enumerate() {
            for char in line.chars() {
                self.buffer[i].push(char);
            }
            self.buffer.push(vec![]);
        }
        Ok(())
    }
    pub fn write_buf_to_file(&self, path: String) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        for line in &self.buffer {
            let mut final_line = vec![];
            let mut buf = [0; 4];
            for char in line {
                char.encode_utf8(&mut buf);
                for byte in buf[0..char.len_utf8()].iter() {
                    final_line.push(*byte);
                }
            }
            '\n'.encode_utf8(&mut buf);
            for byte in buf[0..'\n'.len_utf8()].iter() {
                final_line.push(*byte);
            }
            file.write_all(&final_line)?;
        }
        Ok(())
    }
    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        let buffer = &mut self.buffer;
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
        self.overwrite_char((x, y), c)?;

        execute!(io::stdout(), MoveTo(x + 1, y))?;
        self.last_x = cursor::position()?.0;

        Ok(())
    }
    fn overwrite_char(&mut self, pos: (u16, u16), c: char) -> io::Result<()> {
        let (x, y) = pos;
        let buffer = &mut self.buffer;

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
    pub fn delete_char(&mut self) -> io::Result<()> {
        let (x, y) = cursor::position()?;
        // let deletion_length;
        if x > 0 {
            // deletion_length = self.buffer[y as usize][(x - 1) as usize..self.buffer[y as usize].len()]
            // .iter()
            // .map(|x| x.len_utf8())
            // .sum::<usize>();
            self.buffer[y as usize].remove((x - 1) as usize);
            self.overwrite_char((x-1,y), ' ')?;
            execute!(io::stdout(), MoveTo(x - 1, y))?;
        } else if self.buffer.len() > 1 {
            self.buffer.remove(y as usize);
            if y == 0 {
                execute!(
                    io::stdout(),
                    MoveTo(self.buffer[y as usize].len() as u16, y)
                )?;
                self.write_buf_to_screen(0)?;
            } else {
                execute!(
                    io::stdout(),
                    MoveTo(self.buffer[(y - 1) as usize].len() as u16, y - 1)
                )?;
                self.write_buf_to_screen((y-1) as usize)?;
                
            }
        }

        let (x, y) = cursor::position()?;

        // io::stdout().write_all(vec![b' '; deletion_length].as_ref())?;

        // let mut final_printout = vec![];
        // for char in self.buffer[y as usize][x as usize..self.buffer[y as usize].len()].iter() {
        //     let char_buf = &mut [0; 4];
        //     char.encode_utf8(char_buf);
        //     for byte in char_buf {
        //         final_printout.push(*byte);
        //     }
        // }
        // io::stdout().write_all(&final_printout)?;

        execute!(io::stdout(), MoveTo(x, y))?;

        let (x, _) = cursor::position()?;
        self.last_x = x;

        Ok(())
    }
}

pub enum BufferMode {
    Normal,
    Insert,
}
