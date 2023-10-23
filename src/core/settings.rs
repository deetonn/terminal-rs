use std::{cell::{RefCell, RefMut, Ref}, io, fs::DirEntry, path::Path};
use serde::{Serialize, Deserialize};

use crate::commands::AsStr;

#[derive(Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub trait WithConsoleColor {
    fn with_color(&self, color: Ref<'_, Color>) -> String;
}

impl WithConsoleColor for String {
    fn with_color(&self, color: Ref<'_, Color>) -> String {
        format!("{}{}{}", 
            color.to_ansi_color(),
            self,
            Color::reset())
    }
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r, g, b
        }
    }

    pub fn to_ansi_color(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    pub fn reset() -> String {
        String::from("\x1b[0m")
    }

    pub fn blue() -> Color {
        Self::new(0, 0, 255)
    }

    pub fn light_blue() -> Color {
        Self::new(173, 216, 230)
    }

    pub fn red() -> Color {
        Self::new(255, 0, 0)
    }

    pub fn green() -> Color {
        Self::new(0, 255, 0)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    path: RefCell<String>,
    user_name: RefCell<Option<String>>,
    git_branch: RefCell<Option<String>>,

    path_color: RefCell<Color>,
    user_name_color: RefCell<Color>,
    git_branch_color: RefCell<Color>,
}

#[cfg(windows)]
pub const CONFIG_PATH_DIR_ENVVAR: &'static str = "APPDATA";

#[cfg(not(windows))]
pub const CONFIG_PATH_DIR_ENVVAR: &'static str = "HOME";

pub const CONFIG_DIR_NAME: &'static str = ".term-rs";
pub const SETTINGS_FILE_NAME: &'static str = "settings.json";

pub const DEFAULT_PATH_COLOR: Color = Color::new(20, 255, 247);
pub const DEFAULT_USERNAME_COLOR: Color = Color::new(179, 30, 0);
pub const DEFAULT_GIT_BRANCH_COLOR: Color = Color::new(255, 204, 246);

pub enum SaveError {
    NoSuitablePath(String),
    FailedToSerialize(String),
    IoError(String),
}

impl AsStr for SaveError {
    fn as_str(&self) -> String {
        match self {
            Self::NoSuitablePath(m) => {
                format!("no suitable path: {m}")
            },
            Self::FailedToSerialize(m) => {
                format!("failed to serialize: {m}")
            },
            Self::IoError(m) => {
                format!("IoError: {m}")
            }
        }
    }
}

type PromptOptionMut<'a> = RefMut<'a, Option<String>>;

impl Settings {
    pub fn from_save_or_default(init_path: String) -> Settings {
        if let Some(config) = Self::get_config_folder() {
            // read the file contents then initialize ourself
            // with it.
            let path_to_settings = format!("{}/{}", config, SETTINGS_FILE_NAME);
            let path = Path::new(&path_to_settings);
            let contents = match std::fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(_) => {
                    return Self::new(init_path);
                }
            };
            let deserialized: Self = match serde_json::from_str(&contents) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("failed to deserialize! ({})", e.to_string());
                    return Self::new(init_path);
                }
            };
            return deserialized;
        }
        else {
            Self::new(init_path)
        }
    }

    pub fn new(init_path: String) -> Settings {
        // TODO:
        // get an options parameter to see if the user
        // wants their username/machine name shown.

        Self {
            path: RefCell::new(init_path),
            user_name: RefCell::new(None),
            git_branch: RefCell::new(None),

            path_color: RefCell::new(DEFAULT_PATH_COLOR),
            user_name_color: RefCell::new(DEFAULT_USERNAME_COLOR),
            git_branch_color: RefCell::new(DEFAULT_GIT_BRANCH_COLOR)
        }
    } 

    pub fn get_git_repo(&self) -> PromptOptionMut<'_> {
        self.git_branch.borrow_mut()
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

    pub fn get_path_color(&self) -> RefMut<'_, Color> {
        self.path_color.borrow_mut()
    }

    pub fn get_username_color(&self) -> RefMut<'_, Color> {
        self.user_name_color.borrow_mut()
    }

    pub fn get_branch_color(&self) -> RefMut<'_, Color> {
        self.git_branch_color.borrow_mut()
    }

    fn visit_git_head(&self, file: &Path) -> io::Result<String> {
        // basic parsing of a ".git/HEAD" file.
        // they look like this:
        // "ref: path/to/branch"

        let contents = std::fs::read_to_string(file)?;

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
            Ok(branch.trim().to_string())
        }
        else {
            Err(
                std::io::Error::new(std::io::ErrorKind::InvalidData, ".git/HEAD: had no path to a branch.")
            )
        }
    }

    fn visit_dot_git_folder(&self, entry: &DirEntry) -> io::Result<()> {
        for file in std::fs::read_dir(entry.path())? {
            let file = file?;
            let path = file.path();

            if path.is_file() {
                if path.file_name().unwrap().to_str().unwrap() == "HEAD" {
                    let branch = self.visit_git_head(&path)?; 
                    *self.get_git_repo() = Some(branch);
                }
            }
        }

        Ok(())
    }

    fn handle_git_business(&self) -> io::Result<()> {
        // *path (the path stored in self.path)
        // check if the path or any of its parents
        // contain a ".git" folder.

        // This gets confusing, do we walk back to root?
        // do we keep track of the directory with the ".git"
        // folder and just see if were a sub-directory of it?

        let dir = std::fs::read_dir(&*self.get_path_view())?;
        let mut found_git_folder = false;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.file_name().unwrap() == ".git" {
                self.visit_dot_git_folder(&entry)?;
                found_git_folder = true;
                break
            }
        }

        if !found_git_folder {
            *self.get_git_repo() = None;
        }

        Ok(())
    }

    pub fn build_prompt(&self) -> String {
        let mut result = String::new();
        result.push_str(
            self.path.borrow().with_color(self.path_color.borrow()).as_str()
        );

        if let Some(user_name) = &*self.user_name.borrow() {
            result.push_str(format!("@{}", user_name.with_color(self.user_name_color.borrow())).as_str());
        }

        // dont handle this, it doesnt really matter.
        let _ = self.handle_git_business();

        if let Some(git_repo) = &*self.git_branch.borrow() {
            result.push_str(format!("({})", git_repo.with_color(self.git_branch_color.borrow())).as_str());
        }

        result.push_str("> ");
        result
    }

    /// get the folder where our config folder lives.
    pub fn get_config_location() -> Option<String> {
        match std::env::var(CONFIG_PATH_DIR_ENVVAR) {
            Ok(path) => Some(path),
            Err(_) => {
                None
            }
        }
    }

    // get the actual config folder.
    pub fn get_config_folder() -> Option<String> {
        let location = Self::get_config_location()?;
        let dir = format!("{}/{}", location, CONFIG_DIR_NAME);
        let path = Path::new(&dir);

        if !path.exists() {
            None
        }
        else {
            Some(dir)
        }
    }

    pub fn save(&self) -> Result<(), SaveError> {
        // serialize this class firstly
        let serialized = match serde_json::to_string(&self) {
            Ok(serialized) => serialized,
            Err(e) => {
                eprintln!("failed to serialize settings: {}", e.to_string());
                return Err(
                    SaveError::FailedToSerialize(
                        e.to_string()
                    )
                );
            }
        };

        // find our save location
        let home_dir = match std::env::var(CONFIG_PATH_DIR_ENVVAR) {
            Ok(path) => path,
            Err(e) => {
                return Err(
                    SaveError::NoSuitablePath(e.to_string())
                )
            }
        };

        // concat the config folder with our config directory name
        let full_path = format!("{}/{}", home_dir, CONFIG_DIR_NAME);

        // make sure our config directory exists, otherwise attempt
        // to create it.
        if !Path::new(&full_path).exists() {
            match std::fs::create_dir(&full_path) {
                Ok(_) => (),
                Err(e) => {
                    return Err(
                        SaveError::IoError(
                            e.to_string()
                        )
                    )
                }
            }
        }

        let settings_file_path = format!("{}/{}", full_path, SETTINGS_FILE_NAME);
        let settings_file = Path::new(&settings_file_path);

        match std::fs::write(settings_file, serialized) {
            Ok(_) => {},
            Err(e) => {
                return Err(
                    SaveError::IoError(
                        e.to_string()
                    )
                )
            }
        }

        eprintln!("saved to: {}", settings_file_path);

        Ok(())
    }
}