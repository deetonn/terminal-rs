use std::{cell::Ref, path::Path};
use crate::core::settings::{Color, WithConsoleColor};
use crate::{commands::Cmd, core::Terminal};
use super::{UniError, AsStr};
use is_executable::IsExecutable;
use super::args::ArgInfo;
use termsize::Size;

//macro_rules! println_if {
//    ($cond:expr, $fmt:literal $(,)? $($arg:tt)*) => {{
//        if $cond {
//            println!($fmt, $($arg)*)
//        }
//    }};
//}

pub struct HelpCommand;

impl Cmd for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    fn desc(&self) -> Option<&str> {
        Some("display help information, including commands")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        get a list of all commands that exist.

        flags:
          -b: this will cause it to only display builtin commands.
              without this every file found in the system path will
              also be displayed.
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn super::AsStr>> {
        let info = ArgInfo::new(&args);
        let show_only_builtins = info.has_flag('b');
        for command in ctx.commands().iter() {
            let b = command.borrow();
            let desc = match b.desc() {
                Some(d) => d,
                None => "No description"
            };
            
            if show_only_builtins {
                if b.is_builtin() {
                    println!("{} - {}", b.name(), desc);
                }
            }
            else {
                println!("{} - {}", b.name(), desc);
            }
        }

        Ok(())
    }
}

/// Allow the user to view their input history.
pub struct HistoryCommand;

impl Cmd for HistoryCommand {
    fn desc(&self) -> Option<&str> {
        Some("view your command history")
    }

    fn name(&self) -> &str {
        "history"
    }

    fn docs(&self) -> Option<&str> {
        Some("
        view your command history. this includes things
        you've entered that aren't commands.
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, _args: Vec<&str>) -> Result<(), Box<dyn super::AsStr>> {
        for elem in &*ctx.input().history() {
            eprintln!("{}", elem);
        }
        Ok(())
    }
}

pub struct CdCommand;

pub enum CdError {
    FailedToSetPath(String),
}

impl AsStr for CdError {
    fn as_str(&self) -> String {
        match self {
            Self::FailedToSetPath(msg) => {
                format!("failed to set working directory: {}", msg)
            }
        }
    }
}

impl CdError {
    pub fn boxed(self) -> Box<dyn AsStr> {
        Box::new(self)
    }
}

impl Cmd for CdCommand {
    fn name(&self) -> &str {
        "cd"
    }

    fn desc(&self) -> Option<&str> {
        Some("change the working directory.")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        change the working directory.

        this command accepts all kinds of directory syntax, such
        as \"../\", and \".\".
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn super::AsStr>> {
        // rusts std::env::set_current_dir() function handles stuff like
        // "../" etc... (or the native functions do)

        if args.len() < 1 {
            return Err(
                UniError::TooFewArguments(
                    format!("expected at least one argument, got zero.")
                ).boxed()
            )
        }

        let mut current_path = ctx.settings().get_path();
        let arg = args[0];
        let combined_path = format!("{}/{}", *current_path, arg);

        let path = std::path::Path::new(&combined_path);
        match std::env::set_current_dir(path) {
            Ok(_) => {
                // set the path in the prompt
                // fuck me
                *current_path = std::env::current_dir().unwrap().as_path().as_os_str().to_str().unwrap().to_string();
                Ok(())
            },
            Err(e) => {
                Err(
                    CdError::FailedToSetPath(
                        e.to_string()
                    ).boxed()
                )
            }
        }
    }
}

pub struct LsCommand;

impl Cmd for LsCommand {
    fn name(&self) -> &str {
        "ls"
    }

    fn desc(&self) -> Option<&str> {
        Some("list files and directorys in the current directory.")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        list items in the current directory.

        flags:
          -f: enable filtering
            -X: filter out executable files
            -D: filter out directorys
            -F: filter out regular files
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        let info = ArgInfo::new(&args);
        let working_directory = ctx.current_path();

        let iterator = match std::fs::read_dir(&*working_directory) {
            Ok(o) => o,
            Err(e) => {
                return Err(
                    UniError::IoError(e).boxed()
                )
            }
        };

        let mut items = vec![];

        let has_filter = info.has_flag('f');

        for entry in iterator {
            let entry = match entry {
                Ok(inode) => inode,
                Err(e) => return Err(UniError::IoError(e).boxed())
            };
            let path = entry.path();

            let os_name = entry.file_name();
            let name = os_name.to_str().unwrap().to_string();
            const FILTERED_ITEM: &'static str = "(*)";

            if path.is_executable() {
                if info.has_flag('X') && has_filter {
                    items.push((FILTERED_ITEM.to_string(), format!("(*)")));
                }
                else {
                    items.push((name.clone(), format!("{}", name).rgb(&Color::light_green())));
                }
            }

            if path.is_dir() {
                if info.has_flag('D') && has_filter {
                    items.push((FILTERED_ITEM.to_string(), format!("(*)")));
                }
                else {
                    items.push((name.clone(), format!("{}", name).rgb(&Color::light_blue())));
                }
            }

            if path.is_file() {
                if info.has_flag('F') && has_filter {
                    items.push((FILTERED_ITEM.to_string(), format!("(*)")));
                }
                else {
                    items.push((name.clone(), format!("{}", name).rgb(&Color::light_red())));
                }
            }
        }

        print!("* {} ", "Executable".to_string().rgb(&Color::light_green()));
        print!("* {} ", "Directory".to_string().rgb(&Color::light_blue()));
        print!("* {}", "File".to_string().rgb(&Color::light_red()));
        println!();

        let Size { cols: _, mut rows } = termsize::get().unwrap();
        rows = rows * 6;

        let mut total = 0usize;

        for (original, item) in items {
            total += original.len();

            if total > rows as usize {
                println!();
                total = 0;
            }

            print!("{}   ", item);
        }

        println!();
        Ok(())
    }
}

pub struct ManCommand;

impl Cmd for ManCommand {
    fn name(&self) -> &str {
        "man"
    }

    fn desc(&self) -> Option<&str> {
        Some("view information about specific commands.")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        view documentation about certain commands.

        usage: 
          docs <command>
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        if args.len() < 1 {
            return Err(
                UniError::TooFewArguments(
                    format!("expected the name of a command to show the docs of.")
                ).boxed()
            )
        }

        let name = args[0];
        let commands = ctx.commands();

        let command = match commands.get(name) {
            Some(c) => c,
            None => {
                return Err(
                    UniError::NotFound(
                        format!("the command \"{}\" does not exist.", name)
                    ).boxed()
                )
            }
        };

        println!("( documentation for {} )", name);
        let docs = match command.docs() {
            Some(s) => s,
            None => {
                "this command has no documentation."
            }
        };
        println!("{}", docs);

        Ok(())
    }
}

pub struct ExitCommand;

impl Cmd for ExitCommand {
    fn name(&self) -> &str {
        "exit"
    }

    fn desc(&self) -> Option<&str> {
        Some("quit the application safely, saving all configuration")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        quits the application after saving all configuration.
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, _args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        ctx.quit();

        Ok(())
    }
}

pub struct RmDirCommand;

impl Cmd for RmDirCommand {
    fn name(&self) -> &str {
        "rmdir"
    }

    fn desc(&self) -> Option<&str> {
        Some("remove a directory")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        remove a directory

        flags:
          -f: remove all of the folders child files/folders too.
          \"rmdir <folder> -f\"

        NOTE: all flags must proceed the actual arguments.
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        if args.len() < 1 {
            return Err(
                UniError::TooFewArguments(
                    format!("expected at least one argument: <dir_name>")
                ).boxed()
            )
        }

        let info = ArgInfo::new(&args);
        let remove_all_files_too = info.has_flag('f');
        let cwd = ctx.current_path();
        let requested_folder = args[0].to_string();
        let concated_path = format!("{}/{}", *cwd, requested_folder);

        let path = if Path::new(&requested_folder).exists() {
            Path::new(&requested_folder)
        }
        else {
            // assume its relative
            Path::new(&concated_path)
        };

        if remove_all_files_too {
            match std::fs::remove_dir_all(path) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("failed to remove folder and its contents: ({})", e.to_string());
                }
            }
        }
        else {
            match std::fs::remove_dir(path) {
                Ok(()) => {},
                Err(e) => {
                    eprintln!("failed to remove folder: ({})", e.to_string());
                }
            }
        }

        Ok(())
    }
}

pub struct MkDirCommand;

impl Cmd for MkDirCommand {
    fn name(&self) -> &str {
        "mkdir"
    }

    fn desc(&self) -> Option<&str> {
        Some("create a directory")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        create a directory.

        usage: mkdir <folder_name>
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        if args.len() < 1 {
            return Err(
                UniError::TooFewArguments(
                    format!("expected at least a directory name argument.")
                ).boxed()
            )
        }

        // this .expect() will never happen ^
        let path = args.first().expect("there is no path!").to_string();
        let cwd = ctx.current_path();
        let full_path = format!("{}/{}", *cwd, path);
        let tmp_path = Path::new(&path);

        let dir_to_create = if tmp_path.is_absolute() {
            tmp_path
        }
        else {
            Path::new(&full_path)
        };

        match std::fs::create_dir(dir_to_create) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("failed to create directory: {}", e.to_string())
            }
        };

        Ok(())
    }
}

pub struct ConfigCommand;

const FLAG_COLOR: char = 'c';

const FLAG_PATH: char = 'P';
const FLAG_USERNAME: char = 'U';
const FLAG_BRANCH: char = 'G';

impl ConfigCommand {
    fn parse_color_argument(&self, arg: Option<&&str>) -> Result<u8, Box<dyn AsStr>> {
        if let Some(actual) = arg {
            return match actual.parse::<u8>() {
                Ok(val) => Ok(val),
                Err(e) => {
                    Err(UniError::Custom(
                        format!("could not parse \"{}\" as u8. ({})", actual, e.to_string())
                    ).boxed())
                }
            }
        }
        else {
            Err(
                UniError::Custom(
                    format!("invalid argument format, expected value for color but got nothing.")
                ).boxed()
            )
        }
    }
}

impl Cmd for ConfigCommand {
    fn name(&self) -> &str {
        "cfg"
    }

    fn desc(&self) -> Option<&str> {
        Some("edit your configuration")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        configure your term-rs experience.

        flags:
          -c: This flag tells use you're setting a color config value.
        values:
          -P: You're setting the paths color.
          -U: You're setting the color of your username in the prompt.
          -G: You're setting the color of the github branch (when applicable)

        example:
               R  G B flags
          cfg 255 0 0 -cU <- sets the username color to red.

        NOTE: you cannot set multiple values at a time.
        NOTE: whichever value flag is first will take precedence.
        NOTE: unless you use the \"exit\" command to quit, any changes made
              wont save.
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        if args.len() < 1 {
            return Err(
                UniError::TooFewArguments(
                    format!("{} expects at least one argument.", self.name())
                ).boxed()
            )
        }

        let info = ArgInfo::new(&args);

        if info.has_flag(FLAG_COLOR) {
            let mut col = if info.has_flag(FLAG_PATH) {
                ctx.settings().get_path_color()
            }
            else if info.has_flag(FLAG_USERNAME) {
                ctx.settings().get_username_color()
            }
            else if info.has_flag(FLAG_BRANCH) {
                ctx.settings().get_branch_color()
            }
            else {
                return Err(
                    UniError::Custom(
                        format!("no recognized color flag. (use \"man {}\")", self.name())
                    ).boxed()
                );
            };

            // we expect the arguments to be formatted like this:
            // args[0] = r, args[1] = g, args[2] = b
            let r = self.parse_color_argument(args.get(0))?;
            let g = self.parse_color_argument(args.get(1))?;
            let b = self.parse_color_argument(args.get(2))?;

            let color = Color::new(r, g, b);
            *col = color;

            return Ok(());
        }

        eprintln!("no recognized flags, no work to do.");
        Ok(())
    }
}

pub struct ClearCommand;

impl Cmd for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn desc(&self) -> Option<&str> {
        Some("clear the console buffer")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        clears the console buffer.
        
        very simple command.
        ")
    }

    fn execute(&self, _: Ref<'_, &Terminal>, _: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        print!("\x1B[2J");
        Ok(())
    }
}

pub struct WhereCommand;

impl Cmd for WhereCommand {
    fn name(&self) -> &str {
        "where"
    }

    fn desc(&self) -> Option<&str> {
        Some("find the location of a command")
    }

    fn docs(&self) -> Option<&str> {
        Some("
        find the location of a command.

        usage: where <name>
        ")
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        if args.len() < 1 {
            return Err(
                UniError::TooFewArguments(
                    format!("{} expects at least one argument.", self.name())
                ).boxed()
            )
        }

        let name = args[0];
        match ctx.commands().get(name) {
            Some(cmd) => {
                let path = match cmd.file_location() {
                    Some(p) => p,
                    None => "this command is builtin".to_string()
                };
                println!("{}: {}", name, path);
            },
            None => {
                println!("no such command \"{}\"", name);
            }
        }

        Ok(())
    }
}

pub struct PwdCommand;

