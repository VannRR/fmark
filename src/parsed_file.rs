use std::collections::HashMap;

use crate::bookmark::Bookmark;
use crate::plain_text::PlainText;
use crate::{ADD_BOOKMARK, SEPARATOR_LINE_SYMBOL};

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

    pub fn set_bookmark(
        &mut self,
        plain_text: &mut PlainText,
        new_bookmark: Bookmark,
        old_bookmark: Option<Bookmark>,
    ) {
        if let Some(old_bookmark) = old_bookmark {
            self.remove_bookmark(plain_text, old_bookmark.url());
        }
        if self.add_category(new_bookmark.category().to_string()) {
            plain_text.increment_categories_version();
        };
        let char_count = new_bookmark.title().chars().count();
        if char_count > self.longest_title {
            self.previous_longest_title = self.longest_title;
            self.longest_title = char_count
        }
        self.bookmarks
            .insert(new_bookmark.url().to_string(), new_bookmark);
        plain_text.increment_bookmarks_version();
        plain_text.set_edited_true();
    }

    pub fn remove_bookmark(&mut self, plain_text: &mut PlainText, url: &str) {
        if let Some(bookmark) = self.bookmarks.remove(url) {
            let category = bookmark.category();
            if self.remove_category(category) {
                plain_text.increment_categories_version();
            }
            let char_count = bookmark.title().chars().count();
            if char_count >= self.longest_title {
                self.longest_title = self.previous_longest_title;
            }
            plain_text.increment_bookmarks_version();
            plain_text.set_edited_true();
        }
    }

    pub fn add_category(&mut self, category: String) -> bool {
        if self.category_count.get(&category).is_none() {
            self.category_count.insert(category.clone(), 1);
            if let Err(index) = self.categories.binary_search(&category) {
                let char_count = category.chars().count();
                if char_count > self.longest_category {
                    self.previous_longest_category = self.longest_category;
                    self.longest_category = char_count;
                }

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
                    let char_count = category.chars().count();
                    if char_count >= self.longest_category {
                        self.longest_category = self.previous_longest_category;
                    }

                    self.categories.remove(index);
                    return true;
                }
            } else {
                self.category_count.insert(category.to_string(), count);
            };
        };
        false
    }

    pub fn add_bookmark_option_string(&self) -> String {
        let padding = (self.longest_title + self.longest_category + 8)
            .saturating_sub(ADD_BOOKMARK.chars().count());
        let left_padding = padding / 2;
        let right_padding = padding - left_padding;
        format!(
            "{}{}{}",
            SEPARATOR_LINE_SYMBOL.repeat(left_padding),
            ADD_BOOKMARK,
            SEPARATOR_LINE_SYMBOL.repeat(right_padding)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
    fn test_parsed_file_set_bookmark() {
        let mut plain_text = PlainText::new(PathBuf::from("test.txt"));
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        let bookmark = Bookmark::default();
        let char_count = bookmark.title().chars().count();
        parsed_file.set_bookmark(&mut plain_text, bookmark, None);
        assert_eq!(parsed_file.bookmarks.len(), 1);
        assert_eq!(parsed_file.categories().len(), 1);
        assert!(plain_text.edited());
        assert_eq!(parsed_file.longest_title, char_count);
    }

    #[test]
    fn test_parsed_file_remove_bookmark() {
        let mut plain_text = PlainText::new(PathBuf::from("test.txt"));
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        let bookmark = Bookmark::default();
        let char_count = bookmark.title().chars().count();
        let url = bookmark.url().to_string();
        parsed_file.set_bookmark(&mut plain_text, bookmark, None);
        parsed_file.remove_bookmark(&mut plain_text, &url);
        assert!(parsed_file.bookmarks.is_empty());
        assert!(parsed_file.categories().is_empty());
        assert!(plain_text.edited());
        assert_ne!(parsed_file.longest_title, char_count);
    }

    #[test]
    fn test_parsed_file_add_category() {
        let mut parsed_file = ParsedFile::new("test");
        let category = "test".to_string();
        let char_count = category.chars().count();
        assert!(parsed_file.add_category(category));
        assert_eq!(parsed_file.longest_category, char_count);
    }

    #[test]
    fn test_parsed_file_remove_category() {
        let mut parsed_file = ParsedFile::new("test");
        let category_1 = "test".to_string();
        let char_count_1 = category_1.chars().count();
        let category_2 = "test2".to_string();
        parsed_file.add_category(category_1);
        parsed_file.add_category(category_2.clone());
        assert!(parsed_file.remove_category(category_2.as_str()));
        assert_eq!(parsed_file.longest_category, char_count_1);
    }
}
