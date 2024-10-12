use ncurses::*;

pub struct TerminalBuffer {
    width: usize,
    height: usize,
    front_buffer: Vec<chtype>,
    back_buffer: Vec<chtype>,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        TerminalBuffer {
            width,
            height,
            front_buffer: vec![0; width * height],
            back_buffer: vec![0; width * height],
        }
    }

    pub fn clear(&mut self) {
        self.back_buffer.fill(0);
    }

    pub fn set_char(&mut self, x: usize, y: usize, ch: char, color_pair: i16) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.back_buffer[index] = ch as chtype | COLOR_PAIR(color_pair);
        }
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);
    }

    pub fn render(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                mv(y as i32, x as i32);
                addch(self.front_buffer[index]);
            }
        }
        refresh();
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if self.width == new_width && self.height == new_height {
            return;
        }

        self.width = new_width;
        self.height = new_height;
        let new_size = new_width * new_height;
        
        self.front_buffer.resize(new_size, 0);
        self.back_buffer.resize(new_size, 0);
        self.clear();
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}