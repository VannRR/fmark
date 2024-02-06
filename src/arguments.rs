use std::env;
use std::path::PathBuf;

use crate::Data;

const SUPPORTED_MENU_PROGRAMS: [&str; 3] = ["bemenu", "dmenu", "rofi"];
const ENV_VARIABLE: &str = "BM_DEFAULT_OPTS";
const DEFAULT_MENU_PROGRAM: &str = "bemenu";
const DEFAULT_BROWSER: &str = "firefox";
const DEFAULT_BOOKMARK_FILE_PATH: &str = ".bookmarks";
const DEFAULT_MENU_ROWS: &str = "20";

struct PendingArgs {
    menu_program: Option<String>,
    browser: Option<String>,
    bookmark_file_path: Option<String>,
    menu_rows: Option<String>,
    help: bool,
}

pub struct Arguments {
    menu_program: String,
    browser: String,
    bookmark_file_path: PathBuf,
    menu_rows: String,
}

impl Arguments {
    pub fn new() -> Result<Self, String> {
        let args: Option<Vec<String>> = match env::args().collect::<Vec<String>>() {
            args if args.len() > 1 => Some(args[1..].to_vec()),
            _ => None,
        };
        let user_defaults: Option<Vec<String>> = match env::var(ENV_VARIABLE) {
            Ok(user_defaults) => Some(user_defaults.split(' ').map(|s| s.to_string()).collect()),
            _ => None,
        };
        let pending_values = Self::get_argument_values(args, user_defaults)?;

        if pending_values.help {
            Self::print_help_message();
            std::process::exit(0);
        };

        let menu_program = Self::get_menu_program(pending_values.menu_program)?;
        let browser = Self::get_browser(pending_values.browser)?;
        let bookmark_file_path = Self::get_bookmark_file_path(pending_values.bookmark_file_path)?;
        let menu_rows = Self::get_menu_rows(pending_values.menu_rows);
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

    fn get_argument_values(
        args: Option<Vec<String>>,
        user_defaults: Option<Vec<String>>,
    ) -> Result<PendingArgs, String> {
        let mut p = PendingArgs {
            menu_program: None,
            browser: None,
            bookmark_file_path: None,
            menu_rows: None,
            help: false,
        };

        let process_args = |args: Vec<String>, p: &mut PendingArgs| -> Result<(), String> {
            for i in 0..args.len() {
                if i + 1 < args.len() {
                    match args[i].as_str() {
                        "--menu" | "-m" => p.menu_program = Some(args[i + 1].clone()),
                        "--browser" | "-b" => p.browser = Some(args[i + 1].clone()),
                        "--path" | "-p" => p.bookmark_file_path = Some(args[i + 1].clone()),
                        "--rows" | "-r" => p.menu_rows = Some(args[i + 1].clone()),
                        _ => return Err(Self::unrecognized_arg_message(&args[i])),
                    }
                } else if matches!(args[i].as_str(), "--help" | "-h") {
                    p.help = true;
                }
            }
            Ok(())
        };

        if let Some(user_defaults) = user_defaults {
            process_args(user_defaults, &mut p)?;
        }

        if let Some(args) = args {
            process_args(args, &mut p)?;
        }

        Ok(p)
    }

    fn get_menu_program(menu_program: Option<String>) -> Result<String, String> {
        let menu_program = match menu_program {
            Some(menu_program) => menu_program,
            None => DEFAULT_MENU_PROGRAM.to_string(),
        };

        if SUPPORTED_MENU_PROGRAMS.contains(&menu_program.as_str()) {
            Ok(Self::find_program(&menu_program)?)
        } else {
            Err(format!("Unsupported menu program: {}", menu_program))
        }
    }

    fn get_browser(browser: Option<String>) -> Result<String, String> {
        let browser = match browser {
            Some(browser) => browser,
            None => DEFAULT_BROWSER.to_string(),
        };
        Self::find_program(&browser)
    }

    fn get_bookmark_file_path(path: Option<String>) -> Result<PathBuf, String> {
        match path {
            Some(path) => {
                let custom_path = PathBuf::from(path);
                if custom_path.exists() {
                    Ok(custom_path)
                } else {
                    Err(format!("File not found: {}", custom_path.display()))
                }
            }
            None => {
                let home =
                    env::var("HOME").map_err(|_| "Failed to get HOME environment variable.")?;
                let default_path = PathBuf::from(home).join(DEFAULT_BOOKMARK_FILE_PATH);
                if default_path.exists() {
                    Ok(default_path)
                } else {
                    let template = Data::template();
                    match std::fs::write(&default_path, template) {
                        Ok(_) => Ok(default_path),
                        Err(error) => Err(format!("Failed to create bookmark file: {}", error)),
                    }
                }
            }
        }
    }

    fn get_menu_rows(rows: Option<String>) -> String {
        match rows {
            Some(rows) => {
                let rows = rows.parse::<i32>();
                match rows {
                    Ok(rows) => rows.clamp(1, 255).to_string(),
                    _ => DEFAULT_MENU_ROWS.to_string(),
                }
            }
            None => DEFAULT_MENU_ROWS.to_string(),
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

    #[rustfmt::skip]
    pub fn print_help_message() {
        println!("Usage: bookmarks [OPTIONS]\n");
        println!(
            "This program searches and edits a list of websites from a text file with this format:"
        );
        println!("!!--------|category|-------!!");
        println!("Title # URL\n");
        println!("Options:");
        println!("  -m, --menu            Menu program to use.");
        println!("                        Supported programs are '{}', {}', and '{}'.", 
        SUPPORTED_MENU_PROGRAMS[0], SUPPORTED_MENU_PROGRAMS[1], SUPPORTED_MENU_PROGRAMS[2]);
        println!("                        Default: ({})", DEFAULT_MENU_PROGRAM);
        println!("  -b, --browser         Browser to use.");
        println!("                        Default: ({})", DEFAULT_BROWSER);
        println!("  -p, --path            Path to the bookmark file.");
        println!("                        Default: ($HOME/{})", DEFAULT_BOOKMARK_FILE_PATH);
        println!("  -r, --rows            Number of rows to show in the menu.");
        println!("                        Default: ({})", DEFAULT_MENU_ROWS);
        println!("  -h, --help            Show this help message and exit.\n");
        println!("Environment Variables:");
        println!("BM_DEFAULT_OPTS         Default options");
        println!("                        (e.g. '--menu {} --rows {}')", DEFAULT_MENU_PROGRAM, DEFAULT_MENU_ROWS);
        println!("Please note that the program will check if the specified menu program and browser are found in the PATH. If not, it will fall back to the defaults.");
    }

    fn unrecognized_arg_message(arg: &str) -> String {
        format!("Error: Unrecognized argument '{}'.\nUse '-h, --help' for more information about available options.", arg)
    }
}
