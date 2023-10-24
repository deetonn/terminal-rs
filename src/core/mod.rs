pub mod input;
pub mod settings;

use std::cell::{Cell, RefCell, Ref};
use crate::commands::{Commands, impls::{
    HelpCommand, 
    HistoryCommand, 
    CdCommand, 
    LsCommand, 
    ManCommand, 
    ExitCommand, 
    RmDirCommand, 
    MkDirCommand, 
    ConfigCommand, ClearCommand, WhereCommand, PwdCommand
}, AsStr};
use self::{input::UserInput, settings::Settings};

pub struct Terminal {
    cmds: Commands,
    inp: UserInput,
    _settings: Settings,

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

#[cfg(windows)]
const PATH_ENVVAR_SEP: char = ';';
#[cfg(not(windows))]
const PATH_ENVVAR_SEP: char = ':';

const PATH_ENVIRONMENT_VAR: &'static str = "PATH";

impl Terminal {
    pub fn new() -> Result<Terminal, TerminalInitError> {
        let mut commands = Commands::new();

        commands.push(Box::new(HelpCommand));
        commands.push(Box::new(HistoryCommand));
        commands.push(Box::new(CdCommand));
        commands.push(Box::new(LsCommand));
        commands.push(Box::new(ManCommand));
        commands.push(Box::new(ExitCommand));
        commands.push(Box::new(RmDirCommand));
        commands.push(Box::new(MkDirCommand));
        commands.push(Box::new(ConfigCommand));
        commands.push(Box::new(ClearCommand));
        commands.push(Box::new(WhereCommand));
        commands.push(Box::new(PwdCommand));

        let path = match std::env::var(PATH_ENVIRONMENT_VAR) {
            Ok(path) => Some(path),
            Err(e) => {
                eprintln!("failed to get `PATH` environment variable. ({})", e.to_string());
                None
            }
        };

        if let Some(path) = path {
            let all_directorys: Vec<&str> = path.split(PATH_ENVVAR_SEP).collect();
            for dir in all_directorys {
                match commands.add_path_folder(dir.to_string()) {
                    Ok(()) => {},
                    // TODO: log this information somewhere.
                    Err(_) => ()
                }
            }
        }

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

        let prompt = Settings::from_save_or_default(current_path);
        *prompt.get_user_name() = user_name;

        Ok(Self {
            cmds: commands,
            inp: UserInput::new(),
            _settings: prompt,
            should_quit: Cell::new(false),
        })
    }

    pub fn execute(&self) -> ! {
        let this_ref = RefCell::new(self);

        while !self.should_quit.get() {
            let built_prompt = self.settings().build_prompt();
            let data = self.input().get(built_prompt.as_str());
            match self.commands().try_execute(this_ref.borrow(), data) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("ERROR: {}", e.as_str());
                }
            }
        }

        match self.settings().save() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("ERROR: {}", e.as_str());
            }
        }

        std::process::exit(0);
    }

    pub fn input(&self) -> &UserInput {
        &self.inp
    }

    pub fn quit(&self) {
        self.should_quit.set(true);
    }

    pub fn settings(&self) -> &Settings {
        &self._settings
    }

    pub fn current_path(&self) -> Ref<'_, String> {
        self.settings().get_path_view()
    }

    pub fn commands(&self) -> &Commands {
        &self.cmds
    }
}