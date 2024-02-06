use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const FIELD_SEPARATOR: &str = "@|@";
const TITLE_PADDED_LENGTH: usize = 25;
const CATEGORY_PADDED_LENGTH: usize = 15;
const CATEGORY_SEPARATOR: &str = "-";

#[derive(Clone, PartialEq)]
pub struct Bookmark {
    title: String,
    category: String,
    url: String,
}

impl Bookmark {
    pub fn new(title: String, category: String, url: String) -> Self {
        Self {
            title,
            category,
            url,
        }
    }
    pub fn default() -> Self {
        let title = "A Title".to_string();
        let category = "A Category".to_string();
        let url = " https://website.com".to_string();
        Self {
            title,
            category,
            url,
        }
    }
    pub fn title(&self) -> String {
        self.title.clone()
    }
    pub fn category(&self) -> String {
        self.category.clone()
    }
    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn formatted_line(&self) -> String {
        let title = format!("{:.25}", self.title);
        let category = format!("{:.15}", self.category);
        format!(
            "{:25} {} {:15} {} {}\n",
            title, FIELD_SEPARATOR, category, FIELD_SEPARATOR, self.url
        )
    }
}

struct InvalidLine {
    line: String,
    line_number: usize,
}

impl InvalidLine {
    pub fn new(line: String, line_number: usize) -> Self {
        Self { line, line_number }
    }

    pub fn formatted(&self) -> String {
        let label_one = "original line number";
        let label_two = "invalid line";
        if self.line.contains(label_one) || self.line.contains(label_two) {
            self.line.clone()
        } else {
            format!(
                "{}: {}, {}: {}\n",
                label_one, self.line_number, label_two, self.line
            )
        }
    }
}

pub struct Data {
    file_path: PathBuf,
    plain_text: String,
    previous_version: usize,
    current_version: usize,
    categories: Vec<String>,
    bookmarks: HashMap<String, Bookmark>,
    invalid_lines: Vec<InvalidLine>,
    read: bool,
    edited: bool,
    initialized: bool,
}

impl Data {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            plain_text: String::new(),
            previous_version: 0,
            current_version: 0,
            categories: Vec::new(),
            bookmarks: HashMap::new(),
            invalid_lines: Vec::new(),
            read: false,
            edited: false,
            initialized: false,
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
        self.read = true;
        Ok(())
    }

    pub fn write(&mut self) -> Result<(), String> {
        if !self.read || !self.edited {
            return Ok(());
        }
        self.generate_plain_text();
        fs::write(&self.file_path, &self.plain_text).map_err(|error| {
            format!(
                "Failed to write bookmark file {}: {}",
                self.file_path.display(),
                error
            )
        })
    }

    pub fn categories(&self) -> &Vec<String> {
        &self.categories
    }

    pub fn set_bookmark(&mut self, category: String, title: String, url: String) {
        let title = title.trim().to_string();
        let url = url.trim().to_string();
        let category = category.trim().to_string();
        if !self.categories.contains(&category) {
            self.categories.push(category.clone());
        }
        let old_bookmark = self.bookmarks.get(&url);
        let new_bookmark = Bookmark::new(title, category, url.clone());
        if old_bookmark != Some(&new_bookmark) {
            self.bookmarks.insert(url, new_bookmark);
            self.current_version += 1;
            self.edited = true;
        }
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        if let Some(bookmark) = self.bookmarks.remove(url) {
            let category = bookmark.category();
            if !self.bookmarks.values().any(|b| b.category() == category) {
                self.categories.retain(|c| c != &category);
            }
            self.current_version += 1;
            self.edited = true;
        }
    }

    pub fn generate_plain_text(&mut self) -> &str {
        if self.previous_version == self.current_version && self.initialized {
            return &self.plain_text;
        };

        self.plain_text.clear();

        let mut sorted_categories = self.categories.clone();
        sorted_categories.sort_by(Data::alphabetic_sort);

        for category in sorted_categories {
            let mut bookmarks: Vec<_> = self
                .bookmarks
                .values()
                .filter(|b| b.category() == category)
                .collect();
            bookmarks.sort_by(|a, b| Data::alphabetic_sort(&a.title(), &b.title()));

            for bookmark in bookmarks {
                self.plain_text.push_str(&bookmark.formatted_line());
            }

            let separator = format!(
                "{}\n",
                CATEGORY_SEPARATOR.repeat(
                    TITLE_PADDED_LENGTH + CATEGORY_PADDED_LENGTH + (FIELD_SEPARATOR.len() * 2)
                )
            );
            self.plain_text.push_str(&separator);
        }

        for invalid_line in &self.invalid_lines {
            self.plain_text.push_str(&invalid_line.formatted());
        }

        self.previous_version = self.current_version;
        self.initialized = true;

        &self.plain_text
    }

    pub fn bookmark_from_line(bookmark_line: &str) -> Option<Bookmark> {
        if !bookmark_line.contains(FIELD_SEPARATOR) {
            return None;
        }

        let bookmark = bookmark_line.split(FIELD_SEPARATOR).collect::<Vec<&str>>();

        let title = bookmark[0].trim().to_string();
        let category = bookmark[1].trim().to_string();
        let url = bookmark[2].trim().to_string();

        Some(Bookmark::new(title, category, url))
    }

    fn parse(&mut self) -> Result<(), String> {
        let lines = self.plain_text.lines();
        for (i, line) in lines.enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(CATEGORY_SEPARATOR) {
                continue;
            }

            if line.contains(FIELD_SEPARATOR) {
                if let Some(bookmark) = Data::bookmark_from_line(line) {
                    if !self.categories.contains(&bookmark.category()) {
                        self.categories.push(bookmark.category().clone());
                    }
                    self.bookmarks.insert(bookmark.url.clone(), bookmark);
                }
            } else {
                self.invalid_lines
                    .push(InvalidLine::new(line.to_string(), i + 1));
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
}
