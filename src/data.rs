use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

use crate::bookmark::Bookmark;
use crate::parse_file::*;

pub const SEPARATOR_LINE_SYMBOL: &str = "-";

pub struct Data {
    file_path: PathBuf,
    plain_text: String,
    categories_plain_text: String,
    parsed_file: ParsedFile,
    previous_version: usize,
    current_version: usize,
    edited: bool,
    initialized: bool,
    categories_sorted: bool,
}

impl Data {
    pub fn new(file_path: PathBuf) -> Result<Self, String> {
        let plain_text = fs::read_to_string(&file_path).map_err(|error| {
            format!(
                "Failed to read bookmark file {}: {}",
                file_path.display(),
                error
            )
        })?;
        let parsed_file = ParsedFile::new(&plain_text);
        Ok(Self {
            file_path,
            plain_text,
            categories_plain_text: String::new(),
            parsed_file,
            previous_version: 0,
            current_version: 0,
            edited: false,
            initialized: false,
            categories_sorted: false,
        })
    }

    pub fn write(&mut self) -> Result<(), String> {
        if !self.edited {
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

    pub fn set_bookmark(&mut self, new_bookmark: Bookmark, old_bookmark: Option<Bookmark>) {
        if let Some(old_bookmark) = old_bookmark {
            self.remove_bookmark(old_bookmark.url());
        }
        if !self
            .parsed_file
            .categories
            .contains(&new_bookmark.category().to_string())
        {
            self.parsed_file
                .categories
                .push(new_bookmark.category().to_string());
            self.categories_sorted = false;
        }
        self.parsed_file
            .update_longest_title(new_bookmark.title().to_string().chars().count());
        self.parsed_file
            .update_longest_category(new_bookmark.category().chars().count());
        self.parsed_file
            .bookmarks
            .insert(new_bookmark.url().to_string(), new_bookmark);
        self.current_version += 1;
        self.edited = true;
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        if let Some(bookmark) = self.parsed_file.bookmarks.remove(url) {
            let category = bookmark.category();
            if !self
                .parsed_file
                .bookmarks
                .values()
                .any(|b| b.category() == category)
            {
                self.parsed_file.categories.retain(|c| c != category);
            }
            self.parsed_file
                .revert_longest_title(bookmark.title().chars().count());
            self.parsed_file
                .revert_longest_category(bookmark.category().chars().count());
            self.current_version += 1;
            self.edited = true;
        }
    }

    pub fn plain_text(&mut self) -> &str {
        if self.previous_version == self.current_version && self.initialized {
            return &self.plain_text;
        };

        self.plain_text.clear();

        let mut bookmarks_vec: Vec<_> = self.parsed_file.bookmarks.values().collect();
        let separator_line = format!(
            "{}\n",
            SEPARATOR_LINE_SYMBOL
                .repeat(self.parsed_file.longest_title + self.parsed_file.longest_category + 8)
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
        let combined_len = bookmarks_vec.len() + self.parsed_file.invalid_lines.len();
        for i in 0..combined_len {
            if let Some(line) = self.parsed_file.invalid_lines.get(&i) {
                self.plain_text.push_str(&format!("{}\n", line));
            } else if i < bookmarks_vec.len() {
                if let Some(cat) = current_category {
                    if cat != bookmarks_vec[i].category() {
                        self.plain_text.push_str(&separator_line);
                    }
                }
                current_category = Some(bookmarks_vec[i].category());
                self.plain_text.push_str(&bookmarks_vec[i].to_line(
                    self.parsed_file.longest_title,
                    self.parsed_file.longest_category,
                ));
            }
        }

        self.previous_version = self.current_version;
        self.initialized = true;

        &self.plain_text
    }

    pub fn categories_plain_text(&mut self) -> &str {
        if !self.categories_sorted {
            self.parsed_file
                .categories
                .sort_by(|a, b| Self::alphabetic_sort(a, b));
            self.categories_sorted = true;
            self.categories_plain_text = self
                .parsed_file
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_data_new() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        assert!(Data::new(path).is_ok());
    }

    #[test]
    fn test_data_write() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        let mut data = Data::new(path.clone()).unwrap();
        data.plain_text = "test".to_string();
        data.edited = true;
        assert!(data.write().is_ok());
    }

    #[test]
    fn test_data_set_bookmark() {
        let mut data = Data::new(PathBuf::from("test.txt")).unwrap();
        let bookmark = Bookmark::default();
        data.set_bookmark(bookmark, None);
        assert_eq!(data.parsed_file.bookmarks.len(), 1);
        assert_eq!(data.parsed_file.categories.len(), 1);
        assert!(data.edited);
    }

    #[test]
    fn test_data_remove_bookmark() {
        let mut data = Data::new(PathBuf::from("test.txt")).unwrap();
        let bookmark = Bookmark::default();
        let url = bookmark.url().to_string();
        data.set_bookmark(bookmark, None);
        data.remove_bookmark(&url);
        assert!(data.parsed_file.bookmarks.is_empty());
        assert!(data.parsed_file.categories.is_empty());
        assert!(data.edited);
    }

    #[test]
    fn test_data_plain_text() {
        let mut data = Data::new(PathBuf::from("test.txt")).unwrap();
        let bookmark = Bookmark::default();
        let title_padding = bookmark.title().chars().count();
        let category_padding = bookmark.category().chars().count();
        data.set_bookmark(bookmark.clone(), None);
        assert_eq!(
            data.plain_text(),
            bookmark
                .to_line(title_padding, category_padding)
                .to_string()
        );
    }

    #[test]
    fn test_data_categories_plain_text() {
        let mut data = Data::new(PathBuf::from("test.txt")).unwrap();
        let bookmark = Bookmark::default();
        let category = bookmark.category().to_string();
        data.set_bookmark(bookmark, None);
        assert_eq!(data.categories_plain_text(), format!("{}\n", category));
    }

    #[test]
    fn test_data_alphabetic_sort() {
        assert_eq!(Data::alphabetic_sort("a", "b"), Ordering::Less);
        assert_eq!(Data::alphabetic_sort("b", "a"), Ordering::Greater);
        assert_eq!(Data::alphabetic_sort("a", "a"), Ordering::Equal);
    }
}
