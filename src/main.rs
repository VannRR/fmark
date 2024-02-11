mod arguments;
mod bookmark;
mod data;
mod menu;
mod parsed_file;
mod plain_text;

use arguments::Arguments;
use bookmark::Bookmark;
use data::*;
use menu::*;
use plain_text::ADD_BOOKMARK;

use std::error::Error;
use std::process::Command;

const OPTIONS_GOTO: &str = "goto";
const OPTIONS_MODIFY: &str = "modify";
const OPTIONS_REMOVE: &str = "remove";
const OPTIONS_CANCEL: &str = "cancel";
const OPTIONS: &str = "goto\nmodify\nremove\ncancel";

const TITLE: &str = "title";
const URL: &str = "url";
const CATEGORY: &str = "category";

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::new()?;

    let mut bookmarks_data = Data::new(arguments.bookmark_file_path)?;

    let menu = Menu::new(arguments.menu_program, arguments.menu_rows)?;
    show_list(menu, &mut bookmarks_data, arguments.browser)?;

    bookmarks_data.write()?;

    Ok(())
}

fn show_list(menu: Menu, bookmarks_data: &mut Data, browser: String) -> Result<(), String> {
    let bookmarks_list = Some(bookmarks_data.plain_text_bookmarks());
    let file_line = menu.choose(bookmarks_list, None, "bookmarks")?;
    if file_line.is_empty() {
        return Ok(());
    }

    if let Some(bookmark) = Bookmark::from_line(&file_line) {
        let option = menu.choose(Some(OPTIONS), None, "options")?;
        if option.is_empty() {
            show_list(menu, bookmarks_data, browser)?;
            return Ok(());
        }
        match option.as_str() {
            OPTIONS_GOTO => goto(browser, bookmark.url())?,
            OPTIONS_MODIFY => modify(menu, bookmarks_data, bookmark, browser)?,
            OPTIONS_REMOVE => remove(menu, bookmarks_data, bookmark, browser)?,
            OPTIONS_CANCEL => {
                show_list(menu, bookmarks_data, browser)?;
            }
            _ => (),
        };
    } else if file_line.contains(ADD_BOOKMARK) {
        create(menu, bookmarks_data, browser)?;
    };

    Ok(())
}

fn goto(browser: String, url: &str) -> Result<(), String> {
    Command::new(browser)
        .arg(url)
        .spawn()
        .map_err(|error| format!("Failed to open browser: {}", error))?;

    Ok(())
}

fn create(menu: Menu, bookmarks_data: &mut Data, browser: String) -> Result<(), String> {
    let title = menu.choose(None, None, TITLE)?;
    if title.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let categories = Some(bookmarks_data.plain_text_categories());
    let category = menu.choose(categories, None, CATEGORY)?;
    if category.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let url = menu.choose(None, None, URL)?;
    if url.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let new_bookmark = Bookmark::new(title, category, url);

    bookmarks_data.set_bookmark(new_bookmark, None);

    show_list(menu, bookmarks_data, browser)
}

fn modify(
    menu: Menu,
    bookmarks_data: &mut Data,
    bookmark: Bookmark,
    browser: String,
) -> Result<(), String> {
    let mut title = bookmark.title().to_string();
    title = menu.choose(Some(&title), None, TITLE)?;
    if title.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let categories = Some(bookmarks_data.plain_text_categories());
    let mut category = bookmark.category().to_string();
    category = menu.choose(categories, Some(&category), CATEGORY)?;
    if category.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let mut url = bookmark.url().to_string();
    url = menu.choose(Some(&url), None, URL)?;
    if url.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let new_bookmark = Bookmark::new(title, category, url);

    bookmarks_data.set_bookmark(new_bookmark, Some(bookmark));

    show_list(menu, bookmarks_data, browser)
}

fn remove(
    menu: Menu,
    bookmarks_data: &mut Data,
    bookmark: Bookmark,
    browser: String,
) -> Result<(), String> {
    let prompt = format!("Remove {}? (yes/no)", bookmark.title().trim());
    let answer = menu.choose(None, None, &prompt)?;
    if answer.to_lowercase() != "yes" {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    bookmarks_data.remove_bookmark(bookmark.url());

    show_list(menu, bookmarks_data, browser)
}
