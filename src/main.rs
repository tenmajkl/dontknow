use std::io::{stdin, stdout, Read, Write};

use termios::{tcsetattr, Termios, ECHO, ECHOE, ECHOK, ECHONL, ICANON, IEXTEN, ISIG, OPOST, TCSANOW};

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(PartialEq)]
enum State {
    Continue,
    End,
    Error(String),
}

struct Cursor {
    x: usize,
    y: usize,
}

struct Editor {
    mode: Mode,
    buffer: String,
    text: Vec<String>,
    cursor: Cursor,
}

impl Editor {
    fn handle_input(&mut self, buffer: &[u8]) -> State {
        let input = *buffer.get(0).unwrap() as char;
        
        let (state, mode) = match self.mode {
            Mode::Insert => self.handle_insert(input),
            Mode::Normal => Editor::handle_normal(input),
            Mode::Command => self.handle_command(input),
        };

        self.mode = mode;

        println!("{}", self.text.len());
        print!("\x1b[2J\x1b[0;0H");
        let mut x = 0;
        let mut y = 0;
        for line in &self.text {
            for c in line.split("") {
                print!("{}", c);
                if self.mode == Mode::Insert && x == self.cursor.x && y == self.cursor.y {
                    print!("│");
                }
                x += 1;
            }
            print!("\n");
            y += 1;
            x = 0;
        }
        stdout().lock().flush().unwrap();
        return state;
    }

    fn handle_insert(&mut self, input: char) -> (State, Mode) {
        if input as u8 == 27 {
            return (State::Continue, Mode::Normal);
        }

        if input as u8 == 127 {
            self.text.get_mut(self.cursor.y).unwrap().pop().take();
            if self.cursor.x != 0 {
                self.cursor.x -= 1;
            } else if self.cursor.y != 0 {
                self.cursor.y -= 1;
                self.cursor.x = self.text.get(self.cursor.y).unwrap().len();

                // todo better cursor working 
            }
            return (State::Continue, Mode::Insert);
        }

        if input == '\n' {
            self.cursor.y += 1;
            self.cursor.x = 0;
            self.text.push(String::new());
            return (State::Continue, Mode::Insert);
        }

        self.text.get_mut(self.cursor.y)
            .unwrap()
            .push(input);

        self.cursor.x += 1;
        (State::Continue, Mode::Insert)
    }

    fn handle_normal(input: char) -> (State, Mode) {
        match input {
            'i' => (State::Continue, Mode::Insert),
            ':' => (State::Continue, Mode::Command),
            _ => (State::Continue, Mode::Normal),
        }
    }

    fn handle_command(&mut self, input: char) -> (State, Mode) {
        if input == '\n' {
            return match self.buffer.as_str() {
                "q" | "quit" => (State::End, Mode::Command),
                _ => (State::Error("Unknown command".to_string()), Mode::Normal),
            }
        }

        self.buffer.push(input);
        (State::Continue, Mode::Command)
    }
}

fn main() {
    let old_termios = Termios::from_fd(0).unwrap();
    let mut termios = old_termios.clone();

    termios.c_lflag &= !(ICANON | ECHO | ECHOE | ECHOK | ECHONL | ISIG | IEXTEN);
    tcsetattr(0, TCSANOW, &mut termios).unwrap();
    let mut buff = [0;1];
    let mut input = stdin();
    let mut editor = Editor {
        mode: Mode::Normal,
        buffer: String::new(),
        text: vec!["".to_string()],
        cursor: Cursor { x: 0, y: 0 },
    };
    loop {
        input.read_exact(&mut buff).unwrap(); 
        if editor.handle_input(&buff) == State::End {
            break;
        }
    }

    tcsetattr(0, TCSANOW, &old_termios).unwrap();

}

