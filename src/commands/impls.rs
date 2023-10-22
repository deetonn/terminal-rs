
use std::cell::Ref;

use crate::{commands::Cmd, core::Terminal};

pub struct HelpCommand;

impl Cmd for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }
    fn desc(&self) -> &str {
        "display help information, including commands"
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, _args: Vec<&str>) -> Result<(), Box<dyn super::AsStr>> {
        for command in ctx.commands().iter() {
            let b = command.borrow();
            println!("{} - {}", b.name(), b.desc());
        }

        Ok(())
    }
}

/// Allow the user to view their input history.
pub struct HistoryCommand;

impl Cmd for HistoryCommand {
    fn desc(&self) -> &str {
        "view your command history"
    }

    fn name(&self) -> &str {
        "history"
    }

    fn execute(&self, ctx: Ref<'_, &Terminal>, args: Vec<&str>) -> Result<(), Box<dyn super::AsStr>> {
        for elem in &*ctx.input().history() {
            eprintln!("{}", elem);
        }
        Ok(())
    }
}