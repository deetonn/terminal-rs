
use std::cell::{RefCell, Ref};

use text_io::read;

pub struct UserInput {
    hist: RefCell<Vec<String>>,
}

impl UserInput {
    pub fn new() -> UserInput {
        // TODO: in the future read previous user input
        // from a save file.
        UserInput {
            hist: RefCell::new(Vec::new())
        }
    }

    pub fn get(&self, prompt: &str) -> String {
        print!("{}", prompt);
        let input: String = read!("{}\n");
        self.hist.borrow_mut().push(input.clone());
        input
    }

    pub fn history(&self) -> Ref<'_, Vec<String>> {
        self.hist.borrow()
    }
}