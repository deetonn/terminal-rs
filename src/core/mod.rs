pub mod input;
pub mod prompt;

use std::cell::{Cell, RefCell, Ref};
use crate::commands::{Commands, impls::{HelpCommand, HistoryCommand}};
use self::{input::UserInput, prompt::Prompt};

pub struct Terminal {
    cmds: Commands,
    inp: UserInput,
    _prompt: Prompt,

    // flags (how to bits work??)
    should_quit: Cell<bool>,
}

pub enum TerminalInitError {
    CantGetDirectory(String),
}

impl Into<String> for TerminalInitError {
    fn into(self) -> String {
        match self {
            TerminalInitError::CantGetDirectory(msg) => {
                msg
            }
        }
    }
}

#[cfg(windows)]
const USER_NAME_ENV_NAME: &'static str = "USERNAME";

#[cfg(not(windows))]
const USER_NAME_ENV_NAME: &'static str = "USER";

impl Terminal {
    pub fn new() -> Result<Terminal, TerminalInitError> {
        let mut commands = Commands::new();

        commands.push(Box::new(HelpCommand));
        commands.push(Box::new(HistoryCommand));

        let current_path = match std::env::current_dir() {
            Ok(path) => {
                let p = path.into_os_string();
                p.into_string().unwrap()
            },
            Err(e) => {
                return Err(
                    TerminalInitError::CantGetDirectory(e.to_string())
                );
            }
        };

        // TODO: check if user even wants their name shown.
        // IF we cant find the user name, just dont use one.
        let user_name = match std::env::var(USER_NAME_ENV_NAME) {
            Ok(name) => Some(name),
            Err(_) => {
                None
            }
        };

        let prompt = Prompt::new(current_path);
        *prompt.get_user_name() = user_name;

        Ok(Self {
            cmds: commands,
            inp: UserInput::new(),
            _prompt: prompt,
            should_quit: Cell::new(false),
        })
    }

    pub fn execute(&self) -> ! {
        let this_ref = RefCell::new(self);

        while !self.should_quit.get() {
            let built_prompt = self.prompt().build();
            let data = self.input().get(built_prompt.as_str());
            match self.commands().try_execute(this_ref.borrow(), data) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("ERROR: {}", e.as_str());
                }
            }
        }

        std::process::exit(0);
    }

    pub fn input(&self) -> &UserInput {
        &self.inp
    }

    pub fn quit(&self) {
        self.should_quit.set(false);
    }

    pub fn prompt(&self) -> &Prompt {
        &self._prompt
    }

    pub fn current_path(&self) -> Ref<'_, String> {
        self.prompt().get_path_view()
    }

    pub fn commands(&self) -> &Commands {
        &self.cmds
    }
}