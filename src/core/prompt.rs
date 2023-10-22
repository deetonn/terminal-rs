use std::{cell::{RefCell, RefMut, Ref}, io, fs::DirEntry};

pub struct Prompt {
    path: RefCell<String>,
    user_name: RefCell<Option<String>>,

    git_repo: RefCell<Option<String>>,
}

type PromptOptionMut<'a> = RefMut<'a, Option<String>>;

impl Prompt {
    pub fn new(init_path: String) -> Prompt {
        // TODO:
        // get an options parameter to see if the user
        // wants their username/machine name shown.

        Self {
            path: RefCell::new(init_path),
            user_name: RefCell::new(None),
            git_repo: RefCell::new(None)
        }
    } 

    pub fn get_git_repo(&self) -> PromptOptionMut<'_> {
        self.git_repo.borrow_mut()
    }

    pub fn get_path(&self) -> RefMut<'_, String> {
        self.path.borrow_mut()
    }

    pub fn get_path_view(&self) -> Ref<'_, String> {
        self.path.borrow()
    }

    pub fn get_user_name(&self) -> PromptOptionMut<'_> {
        self.user_name.borrow_mut()
    }

    fn visit_git_head(&self, entry: &DirEntry) -> io::Result<String> {
        // basic parsing of a ".git/HEAD" file.
        // they look like this:
        // "ref: path/to/branch"

        let mut contents = String::new();
        std::fs::read_to_string(&mut contents)?;

        // split the "ref: path/to/branch"
        // into "["ref", "path/to/branch"]"
        let colon_split: Vec<&str> = contents.split(':').collect();
        
        if colon_split.len() != 2 {
            return Err(
                std::io::Error::new(io::ErrorKind::InvalidData, ".git/HEAD: had invalid data inside of it.")
            );
        }

        // now we split the path section, the branch is always
        // the last item.
        let path_to_branch_split: Vec<&str> = colon_split[1].split('/').collect();

        if let Some(branch) = path_to_branch_split.last() {
            Ok(branch.to_string())
        }
        else {
            Err(
                std::io::Error::new(std::io::ErrorKind::InvalidData, ".git/HEAD: had no path to a branch.")
            )
        }
    }

    fn handle_git_business(&self) -> io::Result<()> {
        // *path (the path stored in self.path)
        // check if the path or any of its parents
        // contain a ".git" folder.

        // This gets confusing, do we walk back to root?
        // do we keep track of the directory with the ".git"
        // folder and just see if were a sub-directory of it?

        let dir = std::fs::read_dir(&*self.get_path_view())?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if path.file_name().unwrap() == "HEAD" {
                    let branch = self.visit_git_head(&entry)?; 
                    *self.get_git_repo() = Some(branch);
                }
            }
        }

        Ok(())
    }

    pub fn build(&self) -> String {
        let mut result = String::new();
        result.push_str(self.path.borrow().as_str());

        if let Some(user_name) = &*self.user_name.borrow() {
            result.push_str(format!("@{}", user_name).as_str());
        }

        // dont handle this, it doesnt really matter.
        let _git_result = self.handle_git_business();

        if let Some(git_repo) = &*self.git_repo.borrow() {
            result.push_str(format!("({})", git_repo).as_str());
        }

        result.push_str("> ");
        result
    }
}