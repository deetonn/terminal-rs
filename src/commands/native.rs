
// This file contains stuff for native executables.
// There is a "Cmd" implemenation for executable commands,
// or things that actually have their own binary.

// We iterate through "PATH" and add any executables in those folders
// into a PathLoadedCommand.

use std::{io, path::Path, cell::Ref};
use std::process::{Stdio, Command};
use crate::core::Terminal;
use super::Cmd;

pub trait JustGiveMeTheFuckingName {
    fn get_actual_name(&self) -> String;
}

impl JustGiveMeTheFuckingName for Path {
    fn get_actual_name(&self) -> String {
        self.file_name().unwrap().to_str().unwrap().to_string()
    }
}

pub struct PathLoadedCommand {
    location: String,
    name: String,
    // info: Metadata,
}

impl PathLoadedCommand {
    pub fn new(location: String) -> Result<Self, io::Error> {
        let file = Path::new(&location);
        // the command name is the file name with the extension stripped.
        let command_name = file.get_actual_name();

        // We just need to replace everything after the last "."

        // get all positions of dots.
        let position_of_last_dot: Vec<usize> = command_name
            .chars()
            .enumerate()
            .filter(|(_, p)| if *p == '.' { true } else { false })
            .map(|(offset, _)| offset)
            .collect();

        // use the last dot as our offset, if there are no dots, replace nothing.

        let name = if position_of_last_dot.len() != 0 {
            let last_dot_offset = position_of_last_dot.last().unwrap();
            &command_name[0..=*last_dot_offset - 1]
        }
        else {
            &command_name
        };

        Ok(Self {
            location,
            name: name.to_string()
        })
    }
}

impl Cmd for PathLoadedCommand {
    fn name(&self) -> &str {
        // get the files name, otherwise nothing.
        &self.name
    }

    fn desc(&self) -> Option<&str> {
        None
    }

    fn docs(&self) -> Option<&str> {
        Some("
        This command is a file located on the filesystem.

        custom-options:
          --trs-sandbox: if this flag is present, the command is ran without any environment
                         variables, and isn't inserted into the actual arguments.
        ")
    }

    fn is_builtin(&self) -> bool {
        false
    }

    fn file_location(&self) -> Option<String> {
        Some(self.location.clone())
    }

    fn execute(&self, _ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn super::AsStr>> {
        let mut command = Command::new(&self.location);

        command.stdout(Stdio::inherit());
        command.stdin(Stdio::inherit());
        command.stderr(Stdio::inherit());

        // command.

        for arg in args {
            if arg.to_string() == "--trs-sandbox".to_string() {
                command.env_clear();
                continue
            }
            command.arg(arg);
        }

        let status = match command.status() {
            Ok(status) => status.code().unwrap_or(-1),
            Err(e) => {
                println!("failed to execute command ({})", e.to_string());
                -1
            }
        };

        println!("{} exited with status ({})", self.name(), status);

        Ok(())
    }
}