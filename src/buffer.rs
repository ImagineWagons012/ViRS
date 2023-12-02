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
    pub top: usize,
    pub mode: BufferMode,
    pub path: String,
}

impl BufferContext {
    pub fn write_buf_to_screen(&self, mut from_row: usize) -> io::Result<()> {
        if from_row >= self.buffer.len() {
            from_row = 0;
        }
        let (_, size_y) = (terminal::size()?.0 as usize, terminal::size()?.1 as usize);
        self.clear_screen(None)?;
        for (i, _) in self.buffer.iter().enumerate() {
            if i >= from_row {
                if i > size_y + from_row - 1 {
                    break;
                }
                execute!(io::stdout(), MoveTo(0, (i - from_row) as u16))?;
                self.write_ln(i, None)?;
                execute!(io::stdout(), MoveDown(1), MoveToColumn(0))?;
            }
        }
        Ok(())
    }
    fn clear_ln(&self, from_opt: Option<usize>) -> io::Result<()> {
        let from = from_opt.unwrap_or_default();
        let (size_x, _) = terminal::size()?;
        let mut final_line = vec![];
        let mut buf = [0; 4];
        for i in 0..size_x as usize {
            if i >= from {
                ' '.encode_utf8(&mut buf);
                for byte in buf {
                    final_line.push(byte);
                }
            }
        }
        io::stdout().write_all(&final_line)?;
        Ok(())
    }
    fn clear_screen(&self, from_opt: Option<usize>) -> io::Result<()> {
        let from = from_opt.unwrap_or(0);
        let (size_x, size_y) = terminal::size()?;
        for i in 0..size_y as usize {
            if i >= from {
                execute!(io::stdout(), MoveTo(0, i as u16))?;
                io::stdout().write_all(&vec![b' '; size_x as usize])?;
            }
        }
        Ok(())
    }
    fn write_ln(&self, line_index: usize, from_opt: Option<usize>) -> io::Result<()> {
        let from = from_opt.unwrap_or(0);
        let mut final_line = vec![];
        let mut buf = [0; 4];
        for (i, char) in self.buffer[line_index].iter().enumerate() {
            if i >= from {
                char.encode_utf8(&mut buf);
                for byte in buf[0..char.len_utf8()].iter() {
                    final_line.push(*byte);
                }
            }
        }
        io::stdout().write_all(&final_line)?;
        Ok(())
    }
    pub fn read_file(&mut self, path_opt: Option<String>) -> io::Result<()> {
        let path = path_opt.unwrap_or(self.path.clone());
        let file_string = match read_to_string(path){
            Ok(content) => content,
            Err(_) => return Ok(()),
        };
        let lines = file_string.split('\n').collect::<Vec<&str>>();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                self.buffer.push(vec![]);
            }
            for char in line.chars() {
                if char != '\n' {
                    self.buffer[i].push(char);
                }
            }
        }
        Ok(())
    }
    pub fn write_buf_to_file(&self) -> io::Result<()> {
        let mut file = fs::File::create(self.path.clone())?;
        for (i, line) in self.buffer.iter().enumerate() {
            let mut final_line = vec![];
            let mut buf = [0; 4];
            for char in line {
                char.encode_utf8(&mut buf);
                for byte in buf[0..char.len_utf8()].iter() {
                    final_line.push(*byte);
                }
            }
            if i != self.buffer.len()-1 || !line.is_empty() {
                '\n'.encode_utf8(&mut buf);
                for byte in buf[0..'\n'.len_utf8()].iter() {
                    final_line.push(*byte);
                }
            }
            file.write_all(&final_line)?;
        }
        Ok(())
    }
    pub fn write_char(&mut self, c: char) -> io::Result<()> {
        let buffer = &mut self.buffer;
        let (x, y) = cursor::position()?;
        let buffer_y = self.top + y as usize;
        let (size_x, _size_y) = terminal::size()?;
        if x < size_x - 1 {
            buffer[buffer_y].insert(x as usize, c);
        } else {
            execute!(io::stdout(), MoveTo(0, y + 1))?;
            buffer.push(vec![]);
            buffer[buffer_y + 1].push(c);
        }

        let (x, y) = cursor::position()?;
        self.overwrite_char((x, y), c)?;

        execute!(io::stdout(), MoveTo(x + 1, y))?;
        self.last_x = cursor::position()?.0;

        Ok(())
    }
    fn overwrite_char(&mut self, pos: (u16, u16), c: char) -> io::Result<()> {
        let (_x, y) = pos;
        let buffer_y = self.top + y as usize;

        let char_buf = &mut [0; 4];
        c.encode_utf8(char_buf);

        execute!(io::stdout(), MoveTo(0, y))?;
        self.write_ln(buffer_y, None)?;        

        Ok(())
    }
    pub fn delete_char(&mut self) -> io::Result<()> {
        let (x, y) = cursor::position()?;
        let buffer_y = self.top + y as usize;
        if x > 0 {
            self.buffer[buffer_y].remove((x - 1) as usize);
            self.clear_ln(Some((x - 1) as usize))?;
            execute!(io::stdout(), MoveTo(x - 1, y))?;
            self.write_ln(buffer_y, Some((x - 1) as usize))?;
            execute!(io::stdout(), MoveTo(x - 1, y))?;
        } else if self.buffer.len() > 1 && y != 0 {
            execute!(io::stdout(), MoveTo(x, y - 1))?;

            let x = self.buffer[buffer_y - 1].len() as u16;
            for char in self.buffer[buffer_y].clone() {
                self.buffer[buffer_y - 1].push(char);
            }
            self.buffer.remove(buffer_y);
            self.clear_screen(Some((y - 2) as usize))?;
            self.write_buf_to_screen(self.top)?;
            execute!(io::stdout(), MoveTo(x, y - 1))?;
        }

        let (x, _) = cursor::position()?;
        self.last_x = x;

        Ok(())
    }
}

pub enum BufferMode {
    Normal,
    Insert,
    Highlight,
}
