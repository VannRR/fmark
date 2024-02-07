mod arguments;
mod data;
mod menu;

use arguments::Arguments;
use data::*;
use menu::*;

use std::error::Error;
use std::process::Command;

const OPTIONS_GOTO: &str = "goto";
const OPTIONS_NEW: &str = "create";
const OPTIONS_MODIFY: &str = "modify";
const OPTIONS_REMOVE: &str = "remove";
const OPTIONS: &str = "goto\ncreate\nmodify\nremove";

const TITLE: &str = "title";
const URL: &str = "url";
const CATEGORY: &str = "category";

fn main() -> Result<(), Box<dyn Error>> {
    println!("TODO, add a readme, and upload to github.");

    let arguments = Arguments::new()?;

    let mut bookmarks_data = Data::new(arguments.bookmark_file_path());
    bookmarks_data.read()?;

    let menu = Menu::new(arguments.menu_program(), arguments.menu_rows())?;
    show_list(menu, &mut bookmarks_data, arguments.browser())?;

    bookmarks_data.write()?;

    Ok(())
}

fn show_list(menu: Menu, bookmarks_data: &mut Data, browser: &str) -> Result<(), String> {
    let bookmarks_list = Some(bookmarks_data.plain_text());
    let file_line = menu.choose(bookmarks_list, None, "bookmarks")?;
    if file_line.is_empty() {
        return Ok(());
    }

    if let Some(bookmark) = Data::bookmark_from_line(&file_line) {
        let option = menu.choose(Some(OPTIONS), None, "options")?;
        if option.is_empty() {
            show_list(menu, bookmarks_data, browser)?;
            return Ok(());
        }
        match option.as_str() {
            OPTIONS_NEW => create(menu, bookmarks_data, browser)?,
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

fn create(menu: Menu, bookmarks_data: &mut Data, browser: &str) -> Result<(), String> {
    let title = menu.choose(None, None, TITLE)?;
    if title.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    let categories = Some(bookmarks_data.categories_plain_text());
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

    bookmarks_data.set_bookmark(&category, &title, &url, None);

    show_list(menu, bookmarks_data, browser)
}

fn modify(
    menu: Menu,
    bookmarks_data: &mut Data,
    bookmark: &Bookmark,
    browser: &str,
) -> Result<(), String> {
    let mut title = bookmark.title();
    title = menu.choose(Some(&title), None, TITLE)?;
    if title.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }
    
    let old_url = bookmark.url();
    let mut url = old_url.clone();
    url = menu.choose(Some(&url), None, URL)?;
    if url.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }
    
    let categories = Some(bookmarks_data.categories_plain_text());
    let mut category = bookmark.category();
    category = menu.choose(categories, Some(&category), CATEGORY)?;
    if category.is_empty() {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    bookmarks_data.set_bookmark(&category, &title, &url, Some(&old_url));

    show_list(menu, bookmarks_data, browser)
}

fn remove(
    menu: Menu,
    bookmarks_data: &mut Data,
    bookmark: &Bookmark,
    browser: &str,
) -> Result<(), String> {
    let prompt = format!("Remove {}? (yes/no)", bookmark.title().trim());
    let answer = menu.choose(None, None, &prompt)?;
    if answer.to_lowercase() != "yes" {
        show_list(menu, bookmarks_data, browser)?;
        return Ok(());
    }

    bookmarks_data.remove_bookmark(&bookmark.url());

    show_list(menu, bookmarks_data, browser)
}
