mod arguments;
mod data;
mod menu;

use arguments::Arguments;
use data::*;
use menu::*;

use std::error::Error;
use std::process::Command;

const OPTIONS_NEW: &str = "new";
const OPTIONS_GOTO: &str = "goto";
const OPTIONS_MODIFY: &str = "modify";
const OPTIONS_REMOVE: &str = "remove";
const OPTIONS: &str = "new\ngoto\nmodify\nremove";

const TITLE: &str = "title";
const URL: &str = "url";
const CATEGORY: &str = "category";

fn main() -> Result<(), Box<dyn Error>> {
    println!("TODO, add a readme, add tests, and upload to github.");

    let arguments = Arguments::new()?;

    let mut bookmarks_data = Data::new(arguments.bookmark_file_path());
    bookmarks_data.read()?;

    let menu = Menu::new(arguments.menu_program(), arguments.menu_rows())?;
    show_list(menu, &mut bookmarks_data, arguments.browser())?;

    bookmarks_data.write()?;

    Ok(())
}

fn show_list(menu: Menu, bookmarks_data: &mut Data, browser: &str) -> Result<(), String> {
    let file_line = menu.choose(bookmarks_data.generate_plain_text(), "bookmarks")?;
    if file_line.is_empty() {
        return Ok(());
    }

    if let Some(bookmark) = Data::bookmark_from_line(&file_line) {
        let option = menu.choose(OPTIONS, "options")?;
        if option.is_empty() {
            return Ok(());
        }
        match option.as_str() {
            OPTIONS_NEW => new(menu, bookmarks_data, browser)?,
            OPTIONS_GOTO => goto(browser, &bookmark.url())?,
            OPTIONS_MODIFY => modify(menu, bookmarks_data, &bookmark, browser)?,
            OPTIONS_REMOVE => remove(menu, bookmarks_data, &bookmark, browser)?,
            _ => (),
        };
    };

    Ok(())
}

fn goto(browser: &str, url: &str) -> Result<(), String> {
    Command::new(browser)
        .arg(url)
        .spawn()
        .map_err(|error| format!("Failed to open browser: {}", error))?;

    Ok(())
}

fn new(menu: Menu, bookmarks_data: &mut Data, browser: &str) -> Result<(), String> {
    let answer = menu.input("n", "Add new bookmark? (y/n)")?;
    if answer.to_lowercase() != "y" {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let title = menu.input("", TITLE)?;
    if title.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let categories = bookmarks_data.categories().join("\n");
    let category = menu.choose(&categories, CATEGORY)?;
    if category.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let url = menu.input("", URL)?;
    if url.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    bookmarks_data.set_bookmark(category, title, url);

    show_list(menu, bookmarks_data, browser)
}

fn modify(
    menu: Menu,
    bookmarks_data: &mut Data,
    bookmark: &Bookmark,
    browser: &str,
) -> Result<(), String> {
    let mut title = bookmark.title();
    title = menu.input(&title, TITLE)?;
    if title.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let mut url = bookmark.url();
    url = menu.input(&url, URL)?;
    if url.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }
    let categories = bookmarks_data.categories().join("\n");
    let category = menu.choose(&categories, CATEGORY)?;
    if category.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    bookmarks_data.set_bookmark(category, title, url);

    show_list(menu, bookmarks_data, browser)
}

fn remove(
    menu: Menu,
    bookmarks_data: &mut Data,
    bookmark: &Bookmark,
    browser: &str,
) -> Result<(), String> {
    let answer = menu.input("n", &format!("Remove {}? (y/n)", bookmark.title().trim()))?;
    if answer.to_lowercase() != "y" {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    bookmarks_data.remove_bookmark(&bookmark.url());

    show_list(menu, bookmarks_data, browser)
}
