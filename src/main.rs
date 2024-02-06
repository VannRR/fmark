mod arguments;
mod data;
mod menu;

use arguments::Arguments;
use data::*;
use menu::*;

use std::error::Error;
use std::process::Command;

const OPTIONS_GOTO: &str = "goto";
const OPTIONS_ADD: &str = "add";
const OPTIONS_MODIFY: &str = "modify";
const OPTIONS_REMOVE: &str = "remove";
const OPTIONS_EXIT: &str = "exit";
const OPTIONS: &str = "goto\nadd\nmodify\nremove\nexit";

const SHOW_ALL: &str = "show all";
const SHOW_CATEGORY: &str = "show category";
const SHOW: &str = "show all\nshow category";

const FIELDS_TITLE: &str = "title";
const FIELDS_URL: &str = "url";
const FIELDS_CATEGORY: &str = "category";
const FIELDS: &str = "title\nurl\ncategory";

fn main() -> Result<(), Box<dyn Error>> {
    println!("TODO, add a readme, tui support, and upload to github.");

    let arguments = Arguments::new()?;

    let mut bookmarks_data = Data::new(arguments.bookmark_file_path());
    let menu = Menu::new(arguments.menu_program(), arguments.menu_rows())?;

    let option = menu.choose(OPTIONS, "options")?;
    if option.is_empty() {
        return Ok(());
    }
    match option.as_str() {
        OPTIONS_EXIT => return Ok(()),
        OPTIONS_GOTO => goto(menu, arguments.browser(), &mut bookmarks_data)?,
        OPTIONS_ADD => add(menu, &mut bookmarks_data)?,
        OPTIONS_MODIFY => modify(menu, &mut bookmarks_data)?,
        OPTIONS_REMOVE => remove(menu, &mut bookmarks_data)?,
        _ => (),
    };

    Ok(())
}

fn goto(menu_program: Menu, browser: &str, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let bookmark_line = get_bookmark_line(&menu_program, bookmarks_data)?;
    if bookmark_line.is_empty() {
        return Ok(());
    }

    let fields = match Data::fields_from_line(&bookmark_line) {
        Some(fields) => fields,
        None => return Ok(()),
    };

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

    let title = menu_program.input(FIELDS_TITLE)?;
    if title.is_empty() {
        return Ok(());
    }

    let url = menu_program.input(FIELDS_URL)?;
    if url.is_empty() {
        return Ok(());
    }

    bookmarks_data.set_bookmark(category, title, url, None);
    bookmarks_data.write()
}

fn modify(menu_program: Menu, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let bookmark_line = get_bookmark_line(&menu_program, bookmarks_data)?;
    if bookmark_line.is_empty() {
        return Ok(());
    }

    let fields = match Data::fields_from_line(&bookmark_line) {
        Some(fields) => fields,
        None => return Ok(()),
    };

    let bookmark = bookmarks_data
        .get_bookmark(&fields.url())
        .ok_or(format!("Bookmark not found: {}", fields.url()))?;

    let mut title = bookmark.title();
    let mut category = bookmark.category();

    let url = bookmark.url();
    let mut new_url: Option<String> = None;

    let field = menu_program.choose(FIELDS, "field")?;

    if field == FIELDS_TITLE {
        title = menu_program.input(FIELDS_TITLE)?;
        if title.is_empty() {
            return Ok(());
        }
    } else if field == FIELDS_URL {
        new_url = Some(menu_program.input(FIELDS_URL)?);
        if url.is_empty() {
            return Ok(());
        }
    } else if field == FIELDS_CATEGORY {
        let categories = bookmarks_data.categories().join("\n");
        category = menu_program.choose(&categories, FIELDS_CATEGORY)?;
        if category.is_empty() {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    bookmarks_data.set_bookmark(category, title, url, new_url);
    bookmarks_data.write()
}

fn remove(menu_program: Menu, bookmarks_data: &mut Data) -> Result<(), String> {
    bookmarks_data.read()?;

    let bookmark_line = get_bookmark_line(&menu_program, bookmarks_data)?;
    if bookmark_line.is_empty() {
        return Ok(());
    }

    let fields = match Data::fields_from_line(&bookmark_line) {
        Some(fields) => fields,
        None => return Ok(()),
    };

    let answer = menu_program.input("Are you sure? (y/n)")?;
    if answer.to_lowercase() != "y" {
        return Ok(());
    }

    bookmarks_data.remove_bookmark(&fields.url());
    bookmarks_data.write()
}

fn get_bookmark_line(menu_program: &Menu, bookmarks_data: &Data) -> Result<String, String> {
    let display_type = menu_program.choose(SHOW, "display")?;

    let line = match display_type.as_str() {
        SHOW_ALL => menu_program.choose(bookmarks_data.plain_text(), "bookmarks")?,
        SHOW_CATEGORY => {
            let categories = bookmarks_data.categories().join("\n");
            let category = menu_program.choose(&categories, "categories")?;
            let bookmarks = bookmarks_data.generate_category_plain_text(&category);
            menu_program.choose(&bookmarks, "bookmarks")?
        }
        _ => "".to_string(),
    };

    Ok(line)
}
