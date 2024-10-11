use std::io::{stdin, stdout, Read, Write};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

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

struct Editor {
    mode: Mode,
    buffer: String,
}

impl Editor {
    fn handle_input(&mut self, buffer: &[u8]) -> State {
        let input = *buffer.get(0).unwrap() as char;
        
        let (state, mode) = match self.mode {
            Mode::Insert => Editor::handle_insert(input),
            Mode::Normal => Editor::handle_normal(input),
            Mode::Command => self.handle_command(input),
        };

        self.mode = mode;

        return state;
    }

    fn handle_insert(input: char) -> (State, Mode) {
        if input as u8 == 27 {
            return (State::Continue, Mode::Normal);
        }

        if input as u8 == 127 {
            // tbd visualize whole text field as matrix which then would be operated
            return (State::Continue, Mode::Insert);
        }

        print!("{}", input);
        stdout().lock().flush().unwrap();
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

    termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(0, TCSANOW, &mut termios).unwrap();
    let mut buff = [0;1];
    let mut input = stdin();
    let mut editor = Editor {
        mode: Mode::Normal,
        buffer: String::new(),
    };
    loop {
        input.read_exact(&mut buff).unwrap(); 
        if editor.handle_input(&buff) == State::End {
            break;
        }
    }

    tcsetattr(0, TCSANOW, &old_termios).unwrap();

}

