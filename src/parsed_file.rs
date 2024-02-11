use std::collections::HashMap;

use crate::bookmark::Bookmark;
use crate::plain_text::{PlainText, SEPARATOR_LINE_SYMBOL};

pub struct ParsedFile {
    pub bookmarks: HashMap<String, Bookmark>,
    pub invalid_lines: HashMap<usize, String>,
    category_count: HashMap<String, usize>,
    categories: Vec<String>,
    previous_longest_title: usize,
    pub longest_title: usize,
    previous_longest_category: usize,
    pub longest_category: usize,
}

impl ParsedFile {
    pub fn new(plain_text_bookmarks: &str) -> Self {
        let mut parsed_file = ParsedFile {
            bookmarks: HashMap::new(),
            invalid_lines: HashMap::new(),
            category_count: HashMap::new(),
            categories: Vec::new(),
            previous_longest_title: 0,
            longest_title: 0,
            previous_longest_category: 0,
            longest_category: 0,
        };
        let lines = plain_text_bookmarks.lines();
        for (i, line) in lines.enumerate() {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with(SEPARATOR_LINE_SYMBOL) {
                continue;
            }
            match Bookmark::from_line(trimmed_line) {
                Some(bookmark) => {
                    let title_char_count = bookmark.title().chars().count();
                    if title_char_count > parsed_file.longest_title {
                        parsed_file.previous_longest_title = parsed_file.longest_title;
                        parsed_file.longest_title = title_char_count;
                    }
                    let category_char_count = bookmark.category().chars().count();
                    if category_char_count > parsed_file.longest_category {
                        parsed_file.previous_longest_category = parsed_file.longest_category;
                        parsed_file.longest_category = category_char_count;
                    }
                    let category = bookmark.category().to_string();
                    parsed_file.add_category(category);
                    parsed_file
                        .bookmarks
                        .insert(bookmark.url().to_string(), bookmark);
                }
                None => {
                    parsed_file.invalid_lines.insert(i, line.to_string());
                }
            }
        }
        parsed_file
            .categories
            .sort_by(|a, b| PlainText::alphabetic_sort(a, b));

        parsed_file
    }

    pub fn categories(&self) -> &Vec<String> {
        &self.categories
    }

    pub fn add_category(&mut self, category: String) -> bool {
        if self.category_count.get(&category).is_none() {
            self.category_count.insert(category.clone(), 1);
            if let Err(index) = self.categories.binary_search(&category) {
                self.categories.insert(index, category);
                return true;
            }
        } else {
            let count = self.category_count.get(&category).unwrap() + 1;
            self.category_count.insert(category, count);
        }
        false
    }

    pub fn remove_category(&mut self, category: &str) -> bool {
        if self.category_count.get(category).is_some() {
            let count = self.category_count.get(category).unwrap() - 1;
            if count == 0 {
                if let Ok(index) = self.categories.binary_search(&category.to_string()) {
                    self.categories.remove(index);
                    return true;
                }
            } else {
                self.category_count.insert(category.to_string(), count);
            };
        };
        false
    }

    pub fn update_longest_title(&mut self, title_char_count: usize) {
        if title_char_count > self.longest_title {
            self.previous_longest_title = self.longest_title;
            self.longest_title = title_char_count
        }
    }
    pub fn update_longest_category(&mut self, category_char_count: usize) {
        if category_char_count > self.longest_category {
            self.previous_longest_category = self.longest_category;
            self.longest_category = category_char_count
        }
    }

    pub fn revert_longest_title(&mut self, title_char_count: usize) {
        if title_char_count >= self.longest_title {
            self.longest_title = self.previous_longest_title;
        }
    }

    pub fn revert_longest_category(&mut self, category_char_count: usize) {
        if category_char_count >= self.longest_category {
            self.longest_category = self.previous_longest_category;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_file_new() {
        let default_bookmark = Bookmark::default();
        let title_padding = default_bookmark.title().chars().count();
        let category_padding = default_bookmark.category().chars().count();
        let file = default_bookmark.to_line(title_padding, category_padding);
        let parsed = ParsedFile::new(&file);
        assert_eq!(parsed.bookmarks.len(), 1);
        assert_eq!(parsed.invalid_lines.len(), 0);
        assert_eq!(parsed.longest_title, title_padding);
        assert_eq!(parsed.longest_category, category_padding);
    }

    #[test]
    fn test_parsed_file_add_category() {
        let mut parsed_file = ParsedFile::new("test");
        assert!(parsed_file.add_category("test".to_string()));
    }

    #[test]
    fn test_parsed_file_remove_category() {
        let mut parsed_file = ParsedFile::new("test");
        parsed_file.add_category("test".to_string());
        assert!(parsed_file.remove_category("test"));
    }

    #[test]
    fn test_parsed_file_update_longest_title() {
        let mut parsed_file = ParsedFile::new("test");
        parsed_file.update_longest_title(10);
        assert_eq!(parsed_file.longest_title, 10);
    }

    #[test]
    fn test_parsed_file_update_longest_category() {
        let mut parsed_file = ParsedFile::new("test");
        parsed_file.update_longest_category(10);
        assert_eq!(parsed_file.longest_category, 10);
    }

    #[test]
    fn test_parsed_file_revert_longest_title() {
        let mut parsed_file = ParsedFile::new("test");
        parsed_file.update_longest_title(10);
        parsed_file.update_longest_title(20);
        parsed_file.revert_longest_title(20);
        assert_eq!(parsed_file.longest_title, 10);
    }

    #[test]
    fn test_parsed_file_revert_longest_category() {
        let mut parsed_file = ParsedFile::new("test");
        parsed_file.update_longest_category(10);
        parsed_file.update_longest_category(20);
        parsed_file.revert_longest_category(20);
        assert_eq!(parsed_file.longest_category, 10);
    }
}
