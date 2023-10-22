pub mod core;
pub mod commands;

use crate::core::*;

fn main() {
    match Terminal::new() {
        Ok(inst) => {
            inst.execute();
        },
        Err(e) => {
            let repr: String = e.into();
            eprintln!("ERROR: {}", repr);
        }
    }
}
