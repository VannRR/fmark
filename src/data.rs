use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const TITLE_URL_SEPARATOR: &str = " # ";
const TITLE_PADDED_LENGTH: usize = 45;

const TRUNCATED_INDICATOR: &str = "...";

const CATEGORY_MARK: &str = "!!";
const CATEGORY_PADDING: &str = "-";
const CATEGORY_NAME_FRAME: &str = "|";
const CATEGORY_NAME_MAX_LENGTH: usize =
    TITLE_PADDED_LENGTH - CATEGORY_MARK.len() * 2 - CATEGORY_NAME_FRAME.len() * 2;

#[derive(Clone)]
pub struct Bookmark {
    category: String,
    title: String,
    url: String,
}

impl Bookmark {
    pub fn new(category: String, title: String, url: String) -> Self {
        Self {
            category,
            title,
            url,
        }
    }
    pub fn category(&self) -> String {
        self.category.clone()
    }
    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn formatted_line(&self) -> String {
        format!("{}{}{}\n", self.title, TITLE_URL_SEPARATOR, self.url)
    }
}

pub struct Fields {
    title: String,
    url: String,
}

impl Fields {
    pub fn new(title: String, url: String) -> Self {
        Self { title, url }
    }
    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn url(&self) -> String {
        self.url.clone()
    }
}

pub struct Data {
    file_path: PathBuf,
    plain_text: String,
    categories: Vec<String>,
    bookmarks: HashMap<String, Bookmark>,
}

impl Data {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            plain_text: String::new(),
            categories: Vec::new(),
            bookmarks: HashMap::new(),
        }
    }

    pub fn read(&mut self) -> Result<(), String> {
        self.plain_text = fs::read_to_string(&self.file_path).map_err(|error| {
            format!(
                "Failed to read bookmark file {}: {}",
                self.file_path.display(),
                error
            )
        })?;
        self.parse()?;
        Ok(())
    }

    pub fn write(&self) -> Result<(), String> {
        let plain_text = self.generate_plain_text();
        fs::write(&self.file_path, plain_text).map_err(|error| {
            format!(
                "Failed to write bookmark file {}: {}",
                self.file_path.display(),
                error
            )
        })
    }

    pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    pub fn categories(&self) -> &Vec<String> {
        &self.categories
    }

    pub fn get_bookmark(&self, url: &str) -> Option<&Bookmark> {
        self.bookmarks.get(url)
    }

    pub fn set_bookmark(
        &mut self,
        category: String,
        title: String,
        current_url: String,
        new_url: Option<String>,
    ) {
        let formatted_category = Data::format_category(&category);
        if !self.categories.contains(&formatted_category) {
            self.categories.push(formatted_category.clone());
        }
        let formatted_title = Data::format_title(&title);
        let url = match new_url {
            Some(url) => url,
            None => current_url.clone(),
        };
        let bookmark = Bookmark::new(formatted_category, formatted_title, url);
        self.bookmarks.insert(current_url, bookmark);
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        if let Some(bookmark) = self.bookmarks.remove(url) {
            let category = bookmark.category();
            if !self.bookmarks.values().any(|b| b.category() == category) {
                self.categories.retain(|c| c != &category);
            }
        }
    }

    pub fn generate_plain_text(&self) -> String {
        let mut plain_text = String::new();

        let mut sorted_categories = self.categories.clone();
        sorted_categories.sort_by(Data::alphabetic_sort);

        for category in sorted_categories {
            plain_text.push_str(&format!("{}\n", category));

            let bookmarks = self.generate_category_plain_text(&category);
            plain_text.push_str(&bookmarks);

            plain_text.push('\n');
        }

        plain_text
    }

    pub fn generate_category_plain_text(&self, category: &str) -> String {
        let mut plain_text = String::new();
        plain_text.push_str(&format!("{}\n", category));

        let mut bookmarks: Vec<_> = self
            .bookmarks
            .values()
            .filter(|b| b.category() == category)
            .collect();
        bookmarks.sort_by(|a, b| Data::alphabetic_sort(&a.title(), &b.title()));

        for bookmark in bookmarks {
            plain_text.push_str(&bookmark.formatted_line());
        }

        plain_text
    }

    pub fn format_category(name: &str) -> String {
        let mut name = name.to_string();
        let has_mark = name.starts_with(CATEGORY_MARK);
        let is_right_len = name.len() == TITLE_PADDED_LENGTH;

        if has_mark && is_right_len {
            return name;
        } else if has_mark && !is_right_len {
            name = name
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
        } else if name.len() > CATEGORY_NAME_MAX_LENGTH && !has_mark {
            name.truncate(CATEGORY_NAME_MAX_LENGTH - TRUNCATED_INDICATOR.len());
            name = format!("{}{}", name, TRUNCATED_INDICATOR);
        }

        name = format!("{}{}{}", CATEGORY_NAME_FRAME, name, CATEGORY_NAME_FRAME);

        let mark_chars = CATEGORY_MARK.len() * 2;

        for i in name.len()..TITLE_PADDED_LENGTH - mark_chars {
            if i % 2 == 0 {
                name = format!("{}{}", CATEGORY_PADDING, name);
            } else {
                name = format!("{}{}", name, CATEGORY_PADDING);
            }
        }

        format!("{}{}{}", CATEGORY_MARK, name, CATEGORY_MARK)
    }

    pub fn format_title(title: &str) -> String {
        let mut title = title.to_string();
        if title.starts_with(CATEGORY_MARK) {
            title = title[CATEGORY_MARK.len()..title.len()].to_string();
        } else if title.contains(TITLE_URL_SEPARATOR) {
            title = title.replace(TITLE_URL_SEPARATOR, "");
        };

        match title.len().cmp(&TITLE_PADDED_LENGTH) {
            std::cmp::Ordering::Greater => {
                title.truncate(TITLE_PADDED_LENGTH - TRUNCATED_INDICATOR.len());
                title = format!("{}{}", title, TRUNCATED_INDICATOR);
            }
            std::cmp::Ordering::Less => {
                title = format!("{}{}", title, " ".repeat(TITLE_PADDED_LENGTH - title.len()));
            }
            _ => (),
        }

        title
    }

    pub fn fields_from_line(bookmark_line: &str) -> Option<Fields> {
        if !bookmark_line.contains(TITLE_URL_SEPARATOR) {
            return None;
        }

        let bookmark = bookmark_line
            .split(TITLE_URL_SEPARATOR)
            .collect::<Vec<&str>>();

        let title = Data::format_title(bookmark[0]);
        let url = bookmark[1].to_string();

        Some(Fields::new(title, url))
    }

    fn parse(&mut self) -> Result<(), String> {
        let lines = self.plain_text.lines();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with(CATEGORY_MARK) && line.contains(CATEGORY_MARK) {
                self.categories.push(Data::format_category(line));
            } else if line.contains(TITLE_URL_SEPARATOR) {
                let field = Data::fields_from_line(line);
                let category = self.categories.last().unwrap().clone();

                if let Some(field) = field {
                    self.bookmarks.insert(
                        field.url.clone(),
                        Bookmark::new(category, field.title(), field.url()),
                    );
                }
            }
        }
        Ok(())
    }

    fn alphabetic_sort(a: &String, b: &String) -> std::cmp::Ordering {
        let a = a
            .chars()
            .filter(|c| c.is_ascii_alphabetic() || c.is_ascii_digit())
            .collect::<String>()
            .to_lowercase();
        let b = b
            .chars()
            .filter(|c| c.is_ascii_alphabetic() || c.is_ascii_digit())
            .collect::<String>()
            .to_lowercase();

        let len = if a.len() < b.len() { a.len() } else { b.len() };
        for i in 0..len {
            if a.chars().nth(i) != b.chars().nth(i) {
                return a.chars().nth(i).cmp(&b.chars().nth(i));
            }
        }

        a.len().cmp(&b.len())
    }

    pub fn template() -> String {
        let mut template = String::new();
        template.push_str(&Self::format_category("A Category"));
        template.push('\n');
        template.push_str(&Self::format_title("Project's Github"));
        template.push_str(" https://github.com/vannrr/bookmarks/");
        template
    }
}
