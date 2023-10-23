use std::collections::HashMap;

/// saves information about flags used in the arguments.
pub struct ArgInfo {
    _flags: HashMap<char, bool>,
}

impl ArgInfo {
    pub fn new(args: &Vec<&str>) -> ArgInfo {
        let mut map = HashMap::new();

        for elem in args.iter() {
            if elem.starts_with('-') {
                let all_after_tac = &elem[1..=elem.len()-1];
                for c in all_after_tac.chars() {
                    map.insert(c, true);
                }
            }
        }

        ArgInfo {
            _flags: map
        }
    } 

    pub fn has_flag(&self, flag: char) -> bool {
        self._flags.contains_key(&flag)
    }
}