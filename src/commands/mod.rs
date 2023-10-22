pub mod impls;

use crate::core::Terminal;

use std::cell::{Ref, RefCell, RefMut};

pub trait AsStr {
    fn as_str(&self) -> String;
}

pub enum UniError {
    NotFound(String),
    CommandError(Box<dyn AsStr>),
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
            }
        }
    }
}

pub trait Cmd {
    fn name(&self) -> &str;
    fn desc(&self) -> &str;
    
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