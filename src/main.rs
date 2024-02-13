mod arguments;
mod bookmark;
mod menu;
mod parsed_file;
mod plain_text;

use arguments::Arguments;
use bookmark::Bookmark;
use menu::*;
use parsed_file::ParsedFile;
use plain_text::PlainText;

use std::error::Error;
use std::process::Command;

pub const SEPARATOR_LINE_SYMBOL: &str = "-";
pub const ADD_BOOKMARK: &str = "-| Add Bookmark |-";
pub const TITLE_MAX_LENGTH: usize = 35;
pub const CATEGORY_MAX_LENGTH: usize = 35;

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

    let mut plain_text = PlainText::new(arguments.bookmark_file_path);
    plain_text.read()?;

    let mut parsed_file = ParsedFile::new(plain_text.bookmarks());

    let menu = Menu::new(arguments.menu_program, arguments.menu_rows)?;
    show_list(&mut plain_text, &mut parsed_file, menu, arguments.browser)?;

    plain_text.write(&parsed_file)?;

    Ok(())
}

fn show_list(
    plain_text: &mut PlainText,
    parsed_file: &mut ParsedFile,
    menu: Menu,
    browser: String,
) -> Result<(), String> {
    let add_bookmark_option_string = parsed_file.add_bookmark_option_string();

    plain_text.update_bookmarks(parsed_file);
    let bookmarks_list = Some(plain_text.bookmarks());
    let file_line = menu.choose(
        bookmarks_list,
        Some(&add_bookmark_option_string),
        "bookmarks",
    )?;
    if file_line.is_empty() {
        return Ok(());
    }

    if let Some(bookmark) = Bookmark::from_line(&file_line) {
        let option = menu.choose(Some(OPTIONS), None, "options")?;
        if option.is_empty() {
            show_list(plain_text, parsed_file, menu, browser)?;
            return Ok(());
        }
        match option.as_str() {
            OPTIONS_GOTO => goto(browser, bookmark.url())?,
            OPTIONS_MODIFY => modify(plain_text, parsed_file, menu, browser, bookmark)?,
            OPTIONS_REMOVE => remove(plain_text, parsed_file, menu, browser, bookmark)?,
            OPTIONS_CANCEL => {
                show_list(plain_text, parsed_file, menu, browser)?;
            }
            _ => (),
        };
    } else if file_line.contains(&add_bookmark_option_string) {
        create(plain_text, parsed_file, menu, browser)?;
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

fn create(
    plain_text: &mut PlainText,
    parsed_file: &mut ParsedFile,
    menu: Menu,
    browser: String,
) -> Result<(), String> {
    let title = menu.choose(None, None, TITLE)?;
    if title.is_empty() {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }

    plain_text.update_categories(parsed_file);
    let categories = Some(plain_text.categories());
    let category = menu.choose(categories, None, CATEGORY)?;
    if category.is_empty() {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }

    let url = menu.choose(None, None, URL)?;
    if url.is_empty() {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }

    let new_bookmark = Bookmark::new(title, category, url);

    parsed_file.set_bookmark(plain_text, new_bookmark, None);

    show_list(plain_text, parsed_file, menu, browser)
}

fn modify(
    plain_text: &mut PlainText,
    parsed_file: &mut ParsedFile,
    menu: Menu,
    browser: String,
    bookmark: Bookmark,
) -> Result<(), String> {
    let mut title = bookmark.title().to_string();
    title = menu.choose(Some(&title), None, TITLE)?;
    if title.is_empty() {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }

    let old_category = bookmark.category().to_string();
    let old_category_w_indicator = format!("{} {}", old_category, "<-- current");

    plain_text.update_categories(parsed_file);
    let categories = plain_text
        .categories()
        .replace(&format!("{}\n", old_category), "");

    let mut new_category =
        menu.choose(Some(&categories), Some(&old_category_w_indicator), CATEGORY)?;
    if new_category.is_empty() {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }
    if new_category == old_category_w_indicator {
        new_category = old_category;
    }

    let mut url = bookmark.url().to_string();
    url = menu.choose(Some(&url), None, URL)?;
    if url.is_empty() {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }

    let new_bookmark = Bookmark::new(title, new_category, url);

    parsed_file.set_bookmark(plain_text, new_bookmark, Some(bookmark));

    show_list(plain_text, parsed_file, menu, browser)
}

fn remove(
    plain_text: &mut PlainText,
    parsed_file: &mut ParsedFile,
    menu: Menu,
    browser: String,
    bookmark: Bookmark,
) -> Result<(), String> {
    let prompt = format!("Remove {}? (yes/no)", bookmark.title().trim());
    let answer = menu.choose(None, None, &prompt)?;
    if answer.to_lowercase() != "yes" {
        show_list(plain_text, parsed_file, menu, browser)?;
        return Ok(());
    }

    parsed_file.remove_bookmark(plain_text, bookmark.url());

    show_list(plain_text, parsed_file, menu, browser)
}
