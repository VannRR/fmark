use std::env;
use std::path::PathBuf;

use crate::bookmark::Bookmark;

const SUPPORTED_MENU_PROGRAMS: [&str; 4] = ["bemenu", "dmenu", "rofi", "fzf"];
const ENV_VARIABLE: &str = "FMARK_DEFAULT_OPTS";
const DEFAULT_MENU_PROGRAM: &str = "bemenu";
const DEFAULT_BROWSER: &str = "firefox";
const DEFAULT_BOOKMARK_FILE_PATH: &str = ".bookmarks";
const DEFAULT_MENU_ROWS: &str = "20";

const MENU_ARG_LONG: &str = "--menu";
const MENU_ARG_SHORT: &str = "-m";
const BROWSER_ARG_LONG: &str = "--browser";
const BROWSER_ARG_SHORT: &str = "-b";
const PATH_ARG_LONG: &str = "--path";
const PATH_ARG_SHORT: &str = "-p";
const ROWS_ARG_LONG: &str = "--rows";
const ROWS_ARG_SHORT: &str = "-r";
const HELP_ARG_LONG: &str = "--help";
const HELP_ARG_SHORT: &str = "-h";

struct PendingArgs {
    menu_program: Option<String>,
    browser: Option<String>,
    bookmark_file_path: Option<String>,
    menu_rows: Option<String>,
    help: bool,
}

pub struct Arguments {
    pub menu_program: String,
    pub browser: String,
    pub bookmark_file_path: PathBuf,
    pub menu_rows: String,
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
        let browser = Self::get_browser(pending_values.browser);
        let bookmark_file_path = Self::get_bookmark_file_path(pending_values.bookmark_file_path)?;
        let menu_rows = Self::get_menu_rows(pending_values.menu_rows);
        Ok(Self {
            menu_program,
            browser,
            bookmark_file_path,
            menu_rows,
        })
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
            if args.contains(&HELP_ARG_LONG.to_string())
                || args.contains(&HELP_ARG_SHORT.to_string())
            {
                p.help = true;
                return Ok(());
            }
            for i in (0..args.len() - 1).step_by(2) {
                let arg = args[i].as_str();
                let value = Some(args[i + 1].clone());
                match arg {
                    MENU_ARG_LONG | MENU_ARG_SHORT => p.menu_program = value,
                    BROWSER_ARG_LONG | BROWSER_ARG_SHORT => p.browser = value,
                    PATH_ARG_LONG | PATH_ARG_SHORT => p.bookmark_file_path = value,
                    ROWS_ARG_LONG | ROWS_ARG_SHORT => p.menu_rows = value,
                    _ => return Err(Self::unrecognized_arg_message(arg)),
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
            Ok(menu_program)
        } else {
            Err(format!("Unsupported menu program: {}", menu_program))
        }
    }

    fn get_browser(browser: Option<String>) -> String {
        match browser {
            Some(browser) => browser,
            None => DEFAULT_BROWSER.to_string(),
        }
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
                    let default_bookmark = Bookmark::default();
                    let title_padding = default_bookmark.title().len();
                    let category_padding = default_bookmark.category().len();
                    let template = default_bookmark.to_line(title_padding, category_padding);
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

    #[rustfmt::skip]
    pub fn print_help_message() {
        println!("Usage: fmark [OPTIONS]\n");
        println!(
            "This program can search and modify a formatted plain text list of websites.\n"
        );
        println!("format:");
        println!("  {}\n", Bookmark::default().to_line(0, 0));
        println!("Options:");
        println!("  {}, {:19}Menu program to use.", MENU_ARG_SHORT, MENU_ARG_LONG);
        println!("{:25}Supported programs are '{}'.", "", SUPPORTED_MENU_PROGRAMS.join("', '"));
        println!("{:25}Default: ({})", "", DEFAULT_MENU_PROGRAM);
        println!("  {}, {:19}Browser command URLs will be passed to.", BROWSER_ARG_SHORT, BROWSER_ARG_LONG);
        println!("{:25}Default: ({})", "",DEFAULT_BROWSER);
        println!("  {}, {:19}Path to the bookmark file.", PATH_ARG_SHORT, PATH_ARG_LONG);
        println!("{:25}Default: ($HOME/{})", "", DEFAULT_BOOKMARK_FILE_PATH);
        println!("  {}, {:19}Number of rows to show in the menu.", ROWS_ARG_SHORT, ROWS_ARG_LONG);
        println!("{:25}Default: ({})", "",DEFAULT_MENU_ROWS);
        println!("  {}, {:19}Show this help message and exit.\n", HELP_ARG_SHORT, HELP_ARG_LONG);
        println!("Environment Variables:");
        println!("{:25}Default options", ENV_VARIABLE);
        println!("{:25}(e.g. '--menu {} --rows {}')", "", DEFAULT_MENU_PROGRAM, DEFAULT_MENU_ROWS);
    }

    fn unrecognized_arg_message(arg: &str) -> String {
        format!("Error: Unrecognized argument '{}'. Use '-h, --help' for more information about available options.", arg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_arguments_get_menu_program() {
        // Test with a supported menu program
        let menu_program = Arguments::get_menu_program(Some("bemenu".to_string()));
        assert_eq!(menu_program.unwrap(), "bemenu");

        // Test with an unsupported menu program
        let menu_program = Arguments::get_menu_program(Some("unsupported".to_string()));
        assert!(menu_program.is_err());

        // Test with None, should return the default menu program
        let menu_program = Arguments::get_menu_program(None);
        assert_eq!(menu_program.unwrap(), DEFAULT_MENU_PROGRAM);
    }

    #[test]
    fn test_arguments_get_browser() {
        // Test with a browser
        let browser = Arguments::get_browser(Some("firefox".to_string()));
        assert_eq!(browser, "firefox");

        // Test with None, should return the default browser
        let browser = Arguments::get_browser(None);
        assert_eq!(browser, DEFAULT_BROWSER);
    }

    #[test]
    fn test_arguments_get_bookmark_file_path() {
        // Test with a valid path
        let path = Arguments::get_bookmark_file_path(Some("/home".to_string()));
        assert!(path.is_ok());

        // Test with an invalid path
        let path = Arguments::get_bookmark_file_path(Some("/invalid/path".to_string()));
        assert!(path.is_err());

        // Test with None, should return the default bookmark file path
        let path = Arguments::get_bookmark_file_path(None);
        assert_eq!(
            path.unwrap(),
            PathBuf::from(env::var("HOME").unwrap()).join(DEFAULT_BOOKMARK_FILE_PATH)
        );
    }

    #[test]
    fn test_arguments_get_menu_rows() {
        // Test with a valid number of rows
        let rows = Arguments::get_menu_rows(Some("10".to_string()));
        assert_eq!(rows, "10");

        // Test with an invalid number of rows
        let rows = Arguments::get_menu_rows(Some("invalid".to_string()));
        assert_eq!(rows, DEFAULT_MENU_ROWS);

        // Test with None, should return the default number of menu rows
        let rows = Arguments::get_menu_rows(None);
        assert_eq!(rows, DEFAULT_MENU_ROWS);
    }
}
