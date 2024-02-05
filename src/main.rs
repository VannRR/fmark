mod data;
mod environment;
mod menu;

use data::*;
use std::env;
use environment::Environment;
use menu::*;

use std::error::Error;
use std::process::Command;

const OPTIONS: &str = "goto\nadd\nmodify\nremove\nexit";
const ENTRY_FIELDS: [&str; 3] = ["Title", "URL", "Category"];

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        match arg.as_str() {
            "--help" => {
                print_help_message();
                return Ok(());
             },
            _ => {
                print_unrecognized_arg_message(arg);
                return Ok(());
            },
        }
    }

    let environment = Environment::new()?;

    let mut bookmarks_data = Data::new(environment.bookmark_file_path());
    let menu = Menu::new(environment.menu_program(), environment.menu_rows())?;

    let option = menu.choose(OPTIONS, "options")?;
    if option.is_empty() {
        return Ok(());
    }
    match option.as_str() {
        "exit" => return Ok(()),
        "goto" => goto(menu, environment.browser(), &mut bookmarks_data)?,
        "add" => add(menu, &mut bookmarks_data)?,
        "modify" => modify(menu, &mut bookmarks_data)?,
        "remove" => remove(menu, &mut bookmarks_data)?,
        _ => (),
    };

    Ok(())
}

fn goto(menu_program: Menu, browser: &str, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let bookmark_line = menu_program.choose(bookmarks_data.plain_text(), "bookmarks")?;
    if bookmark_line.is_empty() {
        return Ok(());
    }

    let fields = Data::fields_from_line(&bookmark_line)?;

    Command::new(browser)
        .arg(fields.url())
        .spawn()
        .map_err(|error| format!("Failed to open browser: {}", error))?;

    Ok(())
}

fn add(menu_program: Menu, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let categories = bookmarks_data.categories().join("\n");
    let category = menu_program.choose(&categories, "categories")?;
    if category.is_empty() {
        return Ok(());
    }

    let title = menu_program.input(ENTRY_FIELDS[0])?;
    if title.is_empty() {
        return Ok(());
    }

    let url = menu_program.input(ENTRY_FIELDS[1])?;
    if url.is_empty() {
        return Ok(());
    }

    bookmarks_data.set_bookmark(category, title, url);
    bookmarks_data.write()
}

fn modify(menu_program: Menu, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let bookmark_line = menu_program.choose(bookmarks_data.plain_text(), "bookmarks")?;
    let fields = Data::fields_from_line(&bookmark_line)?;

    let bookmark = bookmarks_data
        .get_bookmark(&fields.url())
        .ok_or(format!("Bookmark not found: {}", fields.url()))?;

    let mut title = bookmark.title();
    let mut url = bookmark.url();
    let mut category = bookmark.category();

    let field = menu_program.choose(&ENTRY_FIELDS.join("\n"), "field")?;

    if field == ENTRY_FIELDS[0] {
        title = menu_program.input(ENTRY_FIELDS[0])?;
        if title.is_empty() {
            return Ok(());
        }
    } else if field == ENTRY_FIELDS[1] {
        url = menu_program.input(ENTRY_FIELDS[1])?;
        if url.is_empty() {
            return Ok(());
        }
    } else if field == ENTRY_FIELDS[2] {
        let categories = bookmarks_data.categories().join("\n");
        category = menu_program.choose(&categories, "categories")?;
        if category.is_empty() {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    bookmarks_data.set_bookmark(category, title, url);
    bookmarks_data.write()
}

fn remove(menu_program: Menu, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let bookmark_line = menu_program.choose(bookmarks_data.plain_text(), "bookmarks")?;
    let fields = Data::fields_from_line(&bookmark_line)?;

    if fields.url().is_empty() {
        return Ok(());
    }

    let answer = menu_program.input("Are you sure? (y/n)")?;

    if answer.to_lowercase() != "y" {
        return Ok(());
    }

    bookmarks_data.remove_bookmark(&fields.url());
    bookmarks_data.write()
}

fn print_help_message() {
    println!("Usage: bookmarks [OPTIONS]\n");
    println!("This program allows you to view and edit a list of web bookmarks from a text file.\n");
    println!("Options:");
    println!("  --help                Show this help message and exit.\n");
    println!("Environment Variables:");
    println!("  BM_MENU_PROGRAM       Set the menu program to use. Supported programs are 'bemenu', 'dmenu', and 'rofi'. Default is 'bemenu'.");
    println!("  BM_BROWSER            Set the browser to open the bookmarks with. Default is 'firefox'.");
    println!("  BM_FILE_PATH          Set the path to the bookmarks file. Default is '.bookmarks' in the home directory.");
    println!("  BM_MENU_ROWS          Set the number of rows in the menu. Default is '10'.\n");
    println!("Please note that the program will check if the specified menu program and browser are found in the PATH. If not, it will fall back to the defaults.");
}

fn print_unrecognized_arg_message(arg: &str) {
    println!("Error: Unrecognized argument '{}'.", arg);
    println!("Use '--help' for more information about available options.");
}
