use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

use crate::parsed_file::*;

pub const SEPARATOR_LINE_SYMBOL: &str = "-";
pub const ADD_BOOKMARK: &str = "-| Add Bookmark |-";
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

pub struct PlainText {
    file_path: PathBuf,
    bookmarks: String,
    previous_bookmarks_version: usize,
    current_bookmarks_version: usize,
    bookmarks_initialized: bool,
    categories: String,
    previous_categories_version: usize,
    current_categories_version: usize,
    categories_initialized: bool,
    edited: bool,
}

impl PlainText {
    pub fn new(file_path: PathBuf) -> Result<Self, String> {
        Ok(Self {
            file_path,
            bookmarks: String::new(),
            previous_bookmarks_version: 0,
            current_bookmarks_version: 0,
            bookmarks_initialized: false,
            categories: String::new(),
            previous_categories_version: 0,
            current_categories_version: 0,
            categories_initialized: false,
            edited: false,
        })
    }

    pub fn bookmarks(&self) -> &str {
        &self.bookmarks
    }

    pub fn categories(&self) -> &str {
        &self.categories
    }

    #[allow(dead_code)]
    pub fn edited(&self) -> bool {
        self.edited
    }

    pub fn set_edited_true(&mut self) {
        self.edited = true;
    }

    pub fn increment_bookmarks_version(&mut self) {
        self.current_bookmarks_version += 1;
    }

    pub fn increment_categories_version(&mut self) {
        self.current_categories_version += 1;
    }

    pub fn read(&mut self) -> Result<(), String> {
        if fs::metadata(&self.file_path)
            .map_err(|error| {
                format!(
                    "Failed to read bookmark file {}: {}",
                    self.file_path.display(),
                    error
                )
            })?
            .len()
            > MAX_FILE_SIZE
        {
            return Err(format!(
                "File larger than {} megabytes: {}",
                MAX_FILE_SIZE / 1_000_000,
                self.file_path.display()
            ));
        }

        self.bookmarks = fs::read_to_string(&self.file_path).map_err(|error| {
            format!(
                "Failed to read bookmark file {}: {}",
                self.file_path.display(),
                error
            )
        })?;

        Ok(())
    }

    pub fn write(&mut self, parsed_file: &ParsedFile) -> Result<(), String> {
        if !self.edited {
            return Ok(());
        }
        self.update_bookmarks(parsed_file);
        fs::write(&self.file_path, &self.bookmarks).map_err(|error| {
            format!(
                "Failed to write bookmark file {}: {}",
                self.file_path.display(),
                error
            )
        })
    }

    pub fn update_bookmarks(&mut self, parsed_file: &ParsedFile) {
        if self.previous_bookmarks_version == self.current_bookmarks_version
            && self.bookmarks_initialized
        {
            return;
        };

        self.bookmarks.clear();

        self.bookmarks
            .push_str(&Self::formatted_add_bookmark(parsed_file));

        let mut bookmarks_vec: Vec<_> = parsed_file.bookmarks.values().collect();
        let separator_line = format!(
            "{}\n",
            SEPARATOR_LINE_SYMBOL
                .repeat(parsed_file.longest_title + parsed_file.longest_category + 8)
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
        let combined_len = bookmarks_vec.len() + parsed_file.invalid_lines.len();
        for i in 0..combined_len {
            if let Some(line) = parsed_file.invalid_lines.get(&i) {
                self.bookmarks.push_str(&format!("{}\n", line));
            } else if i < bookmarks_vec.len() {
                if let Some(cat) = current_category {
                    if cat != bookmarks_vec[i].category() {
                        self.bookmarks.push_str(&separator_line);
                    }
                }
                current_category = Some(bookmarks_vec[i].category());
                self.bookmarks.push_str(
                    &bookmarks_vec[i]
                        .to_line(parsed_file.longest_title, parsed_file.longest_category),
                );
            }
        }

        self.previous_bookmarks_version = self.current_bookmarks_version;
        self.bookmarks_initialized = true;
    }

    pub fn update_categories(&mut self, parsed_file: &ParsedFile) {
        if self.previous_categories_version == self.current_categories_version
            && self.categories_initialized
        {
            return;
        };

        self.categories = parsed_file
            .categories()
            .iter()
            .map(|category| format!("{}\n", category))
            .collect::<String>();

        self.previous_categories_version = self.current_categories_version;
        self.categories_initialized = true;
    }

    pub fn alphabetic_sort(a: &str, b: &str) -> Ordering {
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
        a.cmp(&b)
    }

    fn formatted_add_bookmark(parsed_file: &ParsedFile) -> String {
        let padding = (parsed_file.longest_title + parsed_file.longest_category + 8)
            .saturating_sub(ADD_BOOKMARK.chars().count());
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        format!(
            "{}{}{}\n",
            SEPARATOR_LINE_SYMBOL.repeat(left_padding),
            ADD_BOOKMARK,
            SEPARATOR_LINE_SYMBOL.repeat(right_padding)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::bookmark::Bookmark;

    use super::*;

    #[test]
    fn test_plain_text_new() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        let plain_text = PlainText::new(path);
        assert!(plain_text.is_ok());
    }

    #[test]
    fn test_plain_text_read() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        let mut plain_text = PlainText::new(path).unwrap();
        assert!(plain_text.read().is_ok());
    }

    #[test]
    fn test_plain_text_write() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        let mut plain_text = PlainText::new(path).unwrap();
        let parsed_file = ParsedFile::new(plain_text.bookmarks());
        assert!(plain_text.write(&parsed_file).is_ok());
    }

    #[test]
    fn test_plain_text_update_bookmarks() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        let mut plain_text = PlainText::new(path).unwrap();
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        parsed_file.bookmarks.insert(
            "url".to_string(),
            Bookmark::new(
                "title".to_string(),
                "category".to_string(),
                "url".to_string(),
            ),
        );
        plain_text.update_bookmarks(&parsed_file);
        assert!(!plain_text.bookmarks().is_empty());
    }

    #[test]
    fn test_plain_text_update_categories() {
        let path = PathBuf::from("test.txt");
        let _ = File::create(path.clone()).unwrap();
        let mut plain_text = PlainText::new(path).unwrap();
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        parsed_file.add_category("category".to_string());
        plain_text.update_categories(&parsed_file);
        assert!(!plain_text.categories().is_empty());
    }

    #[test]
    fn test_plain_text_alphabetic_sort() {
        assert!(PlainText::alphabetic_sort("a", "b") == Ordering::Less);
        assert!(PlainText::alphabetic_sort("b", "a") == Ordering::Greater);
        assert!(PlainText::alphabetic_sort("a", "a") == Ordering::Equal);
        assert!(PlainText::alphabetic_sort("a", "A") == Ordering::Equal);
        assert!(PlainText::alphabetic_sort("A", "a") == Ordering::Equal);
        assert!(PlainText::alphabetic_sort("1", "2") == Ordering::Less);
        assert!(PlainText::alphabetic_sort("2", "1") == Ordering::Greater);
        assert!(PlainText::alphabetic_sort("1", "1") == Ordering::Equal);
        assert!(PlainText::alphabetic_sort("1", "a") == Ordering::Less);
    }
}
