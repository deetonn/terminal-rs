pub mod impls;
pub mod args;
pub mod native;

use crate::core::Terminal;

use std::{cell::{Ref, RefCell, RefMut}, io::Error, path::Path};

use self::native::PathLoadedCommand;
use is_executable::IsExecutable;

pub trait AsStr {
    fn as_str(&self) -> String;
}

pub enum UniError {
    NotFound(String),
    CommandError(Box<dyn AsStr>),
    TooFewArguments(String),
    IoError(Error),
    Custom(String),
}

impl UniError {
    pub fn boxed(self) -> Box<dyn AsStr> {
        Box::new(self)
    }
}

impl AsStr for UniError {
    fn as_str(&self) -> String {
        match self {
            UniError::NotFound(info) => {
                format!("NotFound: {}", info)
            },
            UniError::CommandError(e) => {
                format!("CommandErr: {}", e.as_str())
            },
            UniError::TooFewArguments(msg) => {
                format!("not enough arguments: {}", msg)
            },
            UniError::IoError(e) => {
                format!("IoError: {}", e.to_string())
            },
            UniError::Custom(s) => {
                format!("{}", s)
            }
        }
    }
}

pub trait Cmd {
    fn name(&self) -> &str;
    // NOTE: these two are optional because commands loaded
    // from the path dont have descriptions or docs.
    fn desc(&self) -> Option<&str>;
    fn docs(&self) -> Option<&str>;

    fn is_builtin(&self) -> bool {
        true
    }
    fn file_location(&self) -> Option<String> {
        None
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn AsStr>>;
}

pub struct Commands {
    storage: Vec<RefCell<Box<dyn Cmd>>>
}

type Context<'a> = Ref<'a, &'a Terminal>;

impl Commands {
    pub fn new() -> Commands {
        Commands {
            storage: Vec::new()
        }
    }

    pub fn push(&mut self, cmd: Box<dyn Cmd>) {
        self.storage.push(RefCell::new(cmd));
    }

    pub fn get(&self, name: &str) -> Option<Ref<'_, Box<dyn Cmd>>> {
        for elem in &self.storage {
            let b = elem.borrow();
            if b.name() == name {
                return Some(b);
            }
        }
        None
    }

    pub fn get_mut(&self, name: &str) -> Option<RefMut<'_, Box<dyn Cmd>>> {
        for elem in &self.storage {
            let b = elem.borrow();
            if b.name() == name {
                return Some(elem.borrow_mut())
            }
        }
        None
    }

    pub fn execute(&self, ctx: Context<'_>, name: &str, args: Vec<&str>) -> Result<(), Box<dyn AsStr>> {
        if let Some(command) = self.get(name) {
            command.execute(ctx, args)
        }
        else {
            Err(UniError::NotFound(format!("the command {} does not exist.", name)).boxed())
        }
    }

    pub fn add_path_folder(&mut self, path: String) -> std::io::Result<()> {
        let path = Path::new(&path);

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue
            }

            if path.is_executable() {
                let full_path = path.as_os_str().to_str().unwrap().to_string();
                let cmd = PathLoadedCommand::new(full_path)?;
                self.storage.push(RefCell::new(Box::new(cmd)));
            }
        }

        Ok(())
    }

    pub fn try_execute(&self, ctx: Context<'_>, input_data: String) -> Result<(), Box<dyn AsStr>> {
        let parts: Vec<&str> = input_data
            .split(' ')
            .map(|item| item.trim())
            .collect();

        match input_data.len() {
            0 => {
                println!();
                Ok(())
            },
            1 => {
                let first = parts.first().unwrap();
                self.execute(ctx, *first, Vec::new())
            },
            _ => {
                let first = parts.first().unwrap();
                let rest: Vec<&str> = parts[1..=parts.len()-1].into();
                self.execute(ctx, *first, rest)
            }
        }
    }

    pub fn count(&self) -> usize {
        self.storage.len()
    }
    
    pub fn iter(&self) -> std::slice::Iter<'_, RefCell<Box<dyn Cmd>>> {
        self.storage.iter()
    }
}