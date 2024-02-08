use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::parser::*;

pub struct Data {
    file_path: PathBuf,
    plain_text: String,
    bookmarks: HashMap<String, Bookmark>,
    invalid_lines: HashMap<usize, String>,
    categories_plain_text: String,
    categories: Vec<String>,
    previous_version: usize,
    current_version: usize,
    previous_longest_title: usize,
    longest_title: usize,
    previous_longest_category: usize,
    longest_category: usize,
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
            bookmarks: HashMap::new(),
            invalid_lines: HashMap::new(),
            categories_plain_text: String::new(),
            categories: Vec::new(),
            previous_version: 0,
            current_version: 0,
            previous_longest_title: 0,
            longest_title: 0,
            previous_longest_category: 0,
            longest_category: 0,
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
        let parsed_file = parse_file(&self.plain_text);
        self.bookmarks = parsed_file.bookmarks;
        self.invalid_lines = parsed_file.invalid_lines;
        self.previous_longest_title = parsed_file.previous_longest_title;
        self.longest_title = parsed_file.longest_title;
        self.previous_longest_category = parsed_file.previous_longest_category;
        self.longest_category = parsed_file.longest_category;

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

    pub fn set_bookmark(
        &mut self,
        category: String,
        title: String,
        url: String,
        old_url: Option<&str>,
    ) {
        if !self.categories.contains(&category) {
            self.categories.push(category.clone());
            self.categories_sorted = false;
        }
        let old_bookmark = match old_url {
            Some(old_url) => self.bookmarks.get(old_url),
            None => None,
        };
        let new_bookmark = Bookmark::new(title.clone(), category.clone(), url.clone());
        if old_bookmark != Some(&new_bookmark) {
            if let Some(old_url) = old_url {
                self.bookmarks.remove(old_url);
            }
            self.update_longest_title(title.chars().count());
            self.update_longest_category(category.chars().count());
            self.bookmarks.insert(url, new_bookmark);
            self.current_version += 1;
            self.edited = true;
        }
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        if let Some(bookmark) = self.bookmarks.remove(url) {
            let category = bookmark.category();
            if !self.bookmarks.values().any(|b| b.category() == category) {
                self.categories.retain(|c| c != category);
            }
            self.revert_longest_title(bookmark.title().chars().count());
            self.revert_longest_category(bookmark.category().chars().count());
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
        let separator_line = format!(
            "{}\n",
            SEPARATOR_LINE_SYMBOL.repeat(self.longest_title + self.longest_category + 8)
        );

        bookmarks_vec.sort_by(|a, b| {
            let cat_ordering = Self::alphabetic_sort(a.category(), b.category());
            if cat_ordering == Ordering::Equal {
                Self::alphabetic_sort(a.title(), b.title())
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
                        self.plain_text.push_str(&separator_line);
                    }
                }
                current_category = Some(bookmarks_vec[i].category());
                self.plain_text.push_str(
                    &bookmarks_vec[i].formatted_line(self.longest_title, self.longest_category),
                );
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

    fn update_longest_title(&mut self, title_length: usize) {
        if title_length > self.longest_title {
            self.previous_longest_title = self.longest_title;
            self.longest_title = title_length
        }
    }
    fn update_longest_category(&mut self, category_length: usize) {
        if category_length > self.longest_category {
            self.previous_longest_category = self.longest_category;
            self.longest_category = category_length
        }
    }

    fn revert_longest_title(&mut self, title_length: usize) {
        if title_length >= self.longest_title {
            self.longest_title = self.previous_longest_title;
        }
    }

    fn revert_longest_category(&mut self, category_length: usize) {
        if category_length >= self.longest_category {
            self.longest_category = self.previous_longest_category;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_data_read() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        let _ = File::create(&data.file_path).unwrap();
        assert!(data.read().is_ok());
        assert!(data.read);
    }

    #[test]
    fn test_data_write() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        let _ = File::create(&data.file_path).unwrap();
        data.read = true;
        data.edited = true;
        assert!(data.write().is_ok());
    }

    #[test]
    fn test_data_set_bookmark() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.set_bookmark(
            "category".to_string(),
            "title".to_string(),
            "url".to_string(),
            None,
        );
        assert_eq!(data.bookmarks.len(), 1);
        assert_eq!(data.categories.len(), 1);
        assert!(data.edited);
    }

    #[test]
    fn test_data_remove_bookmark() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.set_bookmark(
            "category".to_string(),
            "title".to_string(),
            "url".to_string(),
            None,
        );
        data.remove_bookmark("url");
        assert!(data.bookmarks.is_empty());
        assert!(data.categories.is_empty());
        assert!(data.edited);
    }

    #[test]
    fn test_data_plain_text() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        let bookmark = Bookmark::default();
        let title_padding = bookmark.title().chars().count();
        let category_padding = bookmark.category().chars().count();
        data.set_bookmark(
            bookmark.category().to_string(),
            bookmark.title().to_string(),
            bookmark.url().to_string(),
            None,
        );
        assert_eq!(
            data.plain_text(),
            bookmark
                .formatted_line(title_padding, category_padding)
                .to_string()
        );
    }

    #[test]
    fn test_data_categories_plain_text() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.set_bookmark(
            "category".to_string(),
            "title".to_string(),
            "url".to_string(),
            None,
        );
        assert_eq!(data.categories_plain_text(), "category\n");
    }

    #[test]
    fn test_data_alphabetic_sort() {
        assert_eq!(Data::alphabetic_sort("a", "b"), Ordering::Less);
        assert_eq!(Data::alphabetic_sort("b", "a"), Ordering::Greater);
        assert_eq!(Data::alphabetic_sort("a", "a"), Ordering::Equal);
    }

    #[test]
    fn test_data_update_longest_title() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.update_longest_title(10);
        assert_eq!(data.longest_title, 10);
    }

    #[test]
    fn test_data_update_longest_category() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.update_longest_category(10);
        assert_eq!(data.longest_category, 10);
    }

    #[test]
    fn test_data_revert_longest_title() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.update_longest_title(10);
        data.update_longest_title(20);
        data.revert_longest_title(20);
        assert_eq!(data.longest_title, 10);
    }

    #[test]
    fn test_data_revert_longest_category() {
        let mut data = Data::new(PathBuf::from("test.txt"));
        data.update_longest_category(10);
        data.update_longest_category(20);
        data.revert_longest_category(20);
        assert_eq!(data.longest_category, 10);
    }
}
