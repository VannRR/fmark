use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const FIELD_SEPARATOR_COUNT: usize = 2;
const FIELD_SEPARATOR: &str = "@|@";
const TITLE_PADDED_LENGTH: usize = 24;
const CATEGORY_PADDED_LENGTH: usize = 18;
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

fn separator_line() -> String {
    format!(
        "{}\n",
        CATEGORY_SEPARATOR
            .repeat(TITLE_PADDED_LENGTH + CATEGORY_PADDED_LENGTH + (FIELD_SEPARATOR.len() * 2))
    )
}

pub struct Data {
    file_path: PathBuf,
    plain_text: String,
    categories_plain_text: String,
    previous_version: usize,
    current_version: usize,
    categories: Vec<String>,
    bookmarks: HashMap<String, Bookmark>,
    invalid_lines: HashMap<usize, String>,
    read: bool,
    edited: bool,
    initialized: bool,
    categories_sorted: bool,
}

impl Data {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            plain_text: String::new(),
            categories_plain_text: String::new(),
            previous_version: 0,
            current_version: 0,
            categories: Vec::new(),
            bookmarks: HashMap::new(),
            invalid_lines: HashMap::new(),
            read: false,
            edited: false,
            initialized: false,
            categories_sorted: false,
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
        self.plain_text();
        fs::write(&self.file_path, &self.plain_text).map_err(|error| {
            format!(
                "Failed to write bookmark file {}: {}",
                self.file_path.display(),
                error
            )
        })
    }

    pub fn set_bookmark(&mut self, category: &str, title: &str, url: &str, old_url: Option<&str>) {
        let title = title.trim().to_string();
        let url = url.trim().to_string();
        let category = category.trim().to_string();
        if !self.categories.contains(&category) {
            self.categories.push(category.clone());
            self.categories_sorted = false;
        }
        let old_bookmark = match old_url {
            Some(old_url) => self.bookmarks.get(old_url),
            None => None,
        };
        let new_bookmark = Bookmark::new(title, category.clone(), url.clone());
        if old_bookmark != Some(&new_bookmark) {
            if let Some(old_bookmark) = old_bookmark {
                self.bookmarks.remove(&old_bookmark.url());
            }
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

    pub fn plain_text(&mut self) -> &str {
        if self.previous_version == self.current_version && self.initialized {
            return &self.plain_text;
        };

        self.plain_text.clear();

        let mut bookmarks_vec: Vec<_> = self.bookmarks.values().collect();

        bookmarks_vec.sort_by(|a, b| {
            let cat_ordering = Self::alphabetic_sort(&a.category(), &b.category());
            if cat_ordering == Ordering::Equal {
                Self::alphabetic_sort(&a.title(), &b.title())
            } else {
                cat_ordering
            }
        });

        let mut current_category = None;
        let combined_len = bookmarks_vec.len() + self.invalid_lines.len();
        for i in 0..combined_len {
            if let Some(line) = self.invalid_lines.get(&i) {
                self.plain_text.push_str(line);
            } else if i < bookmarks_vec.len() {
                if let Some(cat) = current_category {
                    if cat != bookmarks_vec[i].category() {
                        self.plain_text.push_str(&separator_line());
                    }
                }
                current_category = Some(bookmarks_vec[i].category());
                self.plain_text.push_str(&bookmarks_vec[i].formatted_line());
            }
        }

        self.previous_version = self.current_version;
        self.initialized = true;

        &self.plain_text
    }

    pub fn categories_plain_text(&mut self) -> &str {
        if !self.categories_sorted {
            self.categories.sort_by(|a, b| Self::alphabetic_sort(a, b));
            self.categories_sorted = true;
            self.categories_plain_text = self
                .categories
                .iter()
                .map(|category| format!("{}\n", category))
                .collect::<String>();
        }
        &self.categories_plain_text
    }

    pub fn bookmark_from_line(line: &str) -> Option<Bookmark> {
        if line.matches(FIELD_SEPARATOR).count() != FIELD_SEPARATOR_COUNT {
            return None;
        }

        let bookmark = line.split(FIELD_SEPARATOR).collect::<Vec<&str>>();

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

            if line.matches(FIELD_SEPARATOR).count() == FIELD_SEPARATOR_COUNT {
                if let Some(bookmark) = Data::bookmark_from_line(line) {
                    if !self.categories.contains(&bookmark.category()) {
                        self.categories.push(bookmark.category().clone());
                    }
                    self.bookmarks.insert(bookmark.url.clone(), bookmark);
                }
            } else {
                self.invalid_lines.insert(i, format!("{}\n", line));
            }
        }

        Ok(())
    }

    fn alphabetic_sort(a: &str, b: &str) -> Ordering {
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
