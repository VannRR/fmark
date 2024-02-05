use std::env;
use std::path::PathBuf;

const SUPPORTED_MENU_PROGRAMS: [&str; 3] = ["bemenu", "dmenu", "rofi"];
const ENV_VARIABLES: [&str; 4] = [
    "BM_MENU_PROGRAM",
    "BM_BROWSER",
    "BM_FILE_PATH",
    "BM_MENU_ROWS",
];
const DEFAULTS: [&str; 4] = ["bemenu", "firefox", ".bookmarks", "10"];

pub struct Environment {
    menu_program: String,
    browser: String,
    bookmark_file_path: PathBuf,
    menu_rows: String,
}

impl Environment {
    pub fn new() -> Result<Self, String> {
        let menu_program = Self::get_menu_program()?;
        let browser = Self::get_browser()?;
        let bookmark_file_path = Self::get_bookmark_file_path()?;
        let menu_rows = Self::get_menu_rows()?;
        Ok(Self {
            menu_program,
            browser,
            bookmark_file_path,
            menu_rows,
        })
    }

    pub fn menu_program(&self) -> &str {
        &self.menu_program
    }

    pub fn browser(&self) -> &str {
        &self.browser
    }

    pub fn bookmark_file_path(&self) -> PathBuf {
        self.bookmark_file_path.clone()
    }

    pub fn menu_rows(&self) -> String {
        self.menu_rows.clone()
    }

    fn get_menu_program() -> Result<String, String> {
        let menu_program = match env::var(ENV_VARIABLES[0]) {
            Ok(program) => program,
            Err(_) => DEFAULTS[0].to_string(),
        };

        if SUPPORTED_MENU_PROGRAMS.contains(&menu_program.as_str()) {
            Ok(Self::find_program(&menu_program)?)
        } else {
            Err(format!("Unsupported menu program: {}", menu_program))
        }
    }

    fn get_browser() -> Result<String, String> {
        match env::var(ENV_VARIABLES[1]) {
            Ok(program) => Ok(Self::find_program(&program)?),
            Err(_) => Ok(Self::find_program(DEFAULTS[1])?),
        }
    }

    fn get_bookmark_file_path() -> Result<PathBuf, String> {
        let path = match env::var(ENV_VARIABLES[2]) {
            Ok(path) => Ok(PathBuf::from(path)),
            Err(_) => match env::var("HOME") {
                Ok(home) => Ok(PathBuf::from(format!("{}/{}", home, DEFAULTS[2]))),
                Err(_) => return Err("Failed to get HOME environment variable.".to_string()),
            },
        };

        path.and_then(|path| {
            if path.exists() {
                Ok(path)
            } else {
                Err("Bookmarks file path does not exist.".to_string())
            }
        })
    }

    fn get_menu_rows() -> Result<String, String> {
        match env::var(ENV_VARIABLES[3]) {
            Ok(rows) => {
                let rows = rows
                    .parse::<u8>()
                    .map(|rows| rows.to_string())
                    .unwrap_or(DEFAULTS[3].to_string());
                Ok(rows)
            }
            Err(_) => Ok(DEFAULTS[3].to_string()),
        }
    }

    fn find_program(name: &str) -> Result<String, String> {
        let paths: Vec<PathBuf> = env::var("PATH")
            .map_err(|error| format!("Failed to get PATH environment variable: {}", error))?
            .split(':')
            .map(PathBuf::from)
            .collect();

        for path in &paths {
            let program_path = path.join(name);
            if program_path.exists() {
                return Ok(name.to_string());
            }
        }

        Err(format!("Program ({}) was not found in the PATH.", name))
    }
}
