extern crate termion;

use termion::{IntoRawMode};
use std::io::{Read, Write, stdout, stdin, Error, ErrorKind};
use std::boxed::Box;
use std::str;

pub struct Expansion {
    entry: String,
    hints: Vec<String>
}

impl Expansion {
    fn new() -> Expansion {
        Expansion {
            entry: String::new(),
            hints: Vec::new()
        }
    }

    fn is_resolved(&self) -> bool {
        self.entry.len() > 0 && self.hints.len() < 2
    }

    fn entry_as_str(&self) -> &str {
        self.entry.as_str()
    }

    fn entry_as_bytes(&self) -> &[u8] {
        self.entry.as_bytes()
    }

    fn hints_as_string(&self) -> String {
        let mut result = String::from("\r\n");
        let joined = self.hints.join("\t");
        result = result + &joined + "\r\n";
        result
    }
}

pub trait Expands {
    fn takes(&self, raw_path: &str) -> Result<bool, Error>;
    fn expand(&mut self, raw_path: &str) -> Result<Expansion, Error>;
}

pub struct TerminalPrompt {
    data: String,
    prompt: String,
    expansions: Vec<Box<Expands>>
}

impl TerminalPrompt {
    pub fn new(prompt: String) -> TerminalPrompt {
        TerminalPrompt {
            data: String::new(),
            prompt: prompt,
            expansions: Vec::new()
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    pub fn register(&mut self, exp: Box<Expands>) {
        self.expansions.push(exp);
    }

    pub fn complete(&mut self) -> Result<Option<Expansion>, Error> {
        for e in &mut self.expansions {
            if !try!(e.takes(self.data.as_str())) {
                continue;
            }

            return Ok(Some(try!(e.expand(self.data.as_str()))));
        }

        Ok(None)
    }

    pub fn done(&mut self) -> String {
        let result = self.data.clone();
        self.data.clear();

        result
    }

    pub fn read_line(&mut self) -> Result<String, Error> {
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();

        let stdin = stdin();
        let stdin = stdin.lock();
        let mut bytes = stdin.bytes();

        try!(stdout.write(self.prompt.as_bytes()));
        try!(stdout.flush());

        loop {
            let b = bytes.next().unwrap().unwrap();

            match b {
                13 => {
                    try!(stdout.write(&[13, 10]));
                    return Ok(self.done());
                },
                27 => Ok(0),
                4 => return Err(Error::new(ErrorKind::UnexpectedEof, "")),
                127 => match self.data.len() > 0 {
                    true => {
                        self.data.pop();
                        stdout.write(&[8, 32, 8])
                    },
                    false => Ok(0)
                },
                9 => match try!(self.complete()) {
                    None => Ok(0),
                    Some(exp) => {
                        if exp.is_resolved() {
                            self.data.push_str(exp.entry_as_str());
                            stdout.write(exp.entry_as_bytes())
                        } else {
                            stdout.write(exp.hints_as_string().as_bytes()).and_then(|_|
                            stdout.write(self.prompt.as_bytes()).and_then(|_|
                            stdout.write(self.data.as_bytes())))
                        }
                    }
                },
                a => {
                    self.data.push(a as char);
                    stdout.write(&[a])
                }
            }.unwrap();

            stdout.flush().unwrap();
        }
    }
}
