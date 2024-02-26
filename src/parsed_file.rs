use std::collections::HashMap;

use crate::bookmark::Bookmark;
use crate::plain_text::PlainText;
use crate::{ADD_BOOKMARK, CATEGORY_MAX_LENGTH, SEPARATOR_LINE_SYMBOL, TITLE_MAX_LENGTH};

pub struct ParsedFile {
    pub bookmarks: HashMap<String, Bookmark>,
    titles_char_count: Vec<usize>,
    pub longest_title: usize,
    pub invalid_lines: HashMap<usize, String>,
    categories: Vec<String>,
    category_count: HashMap<String, usize>,
    categories_char_count: Vec<usize>,
    pub longest_category: usize,
}

impl ParsedFile {
    pub fn new(plain_text_bookmarks: &str) -> Self {
        let mut parsed_file = ParsedFile {
            bookmarks: HashMap::new(),
            titles_char_count: vec![0; TITLE_MAX_LENGTH + 1],
            invalid_lines: HashMap::new(),
            categories: Vec::new(),
            category_count: HashMap::new(),
            categories_char_count: vec![0; CATEGORY_MAX_LENGTH + 1],
            longest_title: 0,
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
                    parsed_file.add_titles_char_count(bookmark.title());
                    parsed_file.add_category(bookmark.category().to_string());
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

    pub fn add_bookmark(&mut self, plain_text: &mut PlainText, new_bookmark: Bookmark) {
        if self.add_category(new_bookmark.category().to_string()) {
            plain_text.increment_categories_version();
        };
        self.add_titles_char_count(new_bookmark.title());
        self.bookmarks
            .insert(new_bookmark.url().to_string(), new_bookmark);
        plain_text.increment_bookmarks_version();
        plain_text.set_edited_true();
    }

    pub fn modify_bookmark(
        &mut self,
        plain_text: &mut PlainText,
        new_bookmark: Bookmark,
        old_bookmark: &Bookmark,
    ) {
        if old_bookmark == &new_bookmark {
            return;
        }

        let old_title = old_bookmark.title();
        let old_category = old_bookmark.category();
        let old_url = old_bookmark.url();
        let new_title = new_bookmark.title();
        let new_category = new_bookmark.category();
        let new_url = new_bookmark.url();

        if old_title != new_title {
            self.remove_titles_char_count(old_title);
            self.add_titles_char_count(new_title);
        }
        if old_category != new_category {
            let removed = self.remove_category(old_category);
            let added = self.add_category(new_category.to_string());
            if removed || added {
                plain_text.increment_categories_version();
            }
        }
        if old_url != new_url {
            self.bookmarks.remove(old_url);
            self.bookmarks.insert(new_url.to_string(), new_bookmark);
        } else {
            self.bookmarks.insert(old_url.to_string(), new_bookmark);
        }
        plain_text.increment_bookmarks_version();
        plain_text.set_edited_true();
    }

    pub fn remove_bookmark(&mut self, plain_text: &mut PlainText, url: &str) {
        if let Some(bookmark) = self.bookmarks.remove(url) {
            let category = bookmark.category();
            if self.remove_category(category) {
                plain_text.increment_categories_version();
            }
            self.remove_titles_char_count(bookmark.title());
            plain_text.increment_bookmarks_version();
            plain_text.set_edited_true();
        }
    }

    pub fn add_category(&mut self, category: String) -> bool {
        match self.category_count.get_mut(&category) {
            Some(count) => {
                *count += 1;
            }
            None => {
                self.category_count.insert(category.clone(), 1);
                if let Err(index) = self.categories.binary_search(&category) {
                    self.add_category_char_count(&category);
                    self.categories.insert(index, category);
                    return true;
                }
            }
        }
        false
    }

    pub fn remove_category(&mut self, category: &str) -> bool {
        if let Some(count) = self.category_count.get_mut(category) {
            *count -= 1;
            if *count == 0 {
                if let Ok(index) = self.categories.binary_search(&category.to_string()) {
                    self.remove_category_char_count(category);
                    self.categories.remove(index);
                    return true;
                }
            }
        };
        false
    }

    pub fn add_bookmark_option_string(&self) -> String {
        let padding = (self.longest_title + self.longest_category + 11)
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

    fn add_titles_char_count(&mut self, title: &str) {
        Self::add_char_count(&mut self.titles_char_count, &mut self.longest_title, title);
    }

    fn remove_titles_char_count(&mut self, title: &str) {
        Self::remove_char_count(&mut self.titles_char_count, &mut self.longest_title, title);
    }

    fn add_category_char_count(&mut self, category: &str) {
        Self::add_char_count(
            &mut self.categories_char_count,
            &mut self.longest_category,
            category,
        );
    }

    fn remove_category_char_count(&mut self, category: &str) {
        Self::remove_char_count(
            &mut self.categories_char_count,
            &mut self.longest_category,
            category,
        );
    }

    fn add_char_count(char_count_vec: &mut [usize], longest: &mut usize, field: &str) {
        let char_count = field.chars().count();
        if char_count == 0 {
            return;
        }

        if char_count > *longest {
            *longest = char_count;
        }

        if let Some(amount) = char_count_vec.get_mut(char_count) {
            *amount += 1;
        }
    }

    fn remove_char_count(char_count_vec: &mut [usize], longest: &mut usize, field: &str) {
        let char_count = field.chars().count();
        if char_count == 0 {
            return;
        }
        if let Some(amount) = char_count_vec.get_mut(char_count) {
            *amount -= 1;
            if *amount == 0 && char_count == *longest {
                for i in (1..=char_count).rev() {
                    if let Some(amount) = char_count_vec.get(i) {
                        if *amount > 0 {
                            *longest = i;
                            return;
                        }
                    }
                }
                *longest = 0;
            }
        }
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
    fn test_parsed_file_add_bookmark() {
        let mut plain_text = PlainText::new(PathBuf::from("test.txt"));
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        let bookmark = Bookmark::default();
        let char_count = bookmark.title().chars().count();
        parsed_file.add_bookmark(&mut plain_text, bookmark);
        assert_eq!(parsed_file.bookmarks.len(), 1);
        assert_eq!(parsed_file.categories().len(), 1);
        assert!(plain_text.edited());
        assert_eq!(parsed_file.longest_title, char_count);
    }

    #[test]
    fn test_parsed_file_modify_bookmark() {
        let mut plain_text = PlainText::new(PathBuf::from("test.txt"));
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        let old_bookmark = Bookmark::default();
        parsed_file.add_bookmark(&mut plain_text, old_bookmark.clone());
        let title = "new title".to_string();
        let category = "new category".to_string();
        let url = "new url".to_string();
        let new_bookmark = Bookmark::new(url, title, category);
        parsed_file.modify_bookmark(&mut plain_text, new_bookmark.clone(), &old_bookmark);
        assert_eq!(parsed_file.bookmarks.len(), 1);
        assert_eq!(parsed_file.categories().len(), 1);
        assert!(plain_text.edited());
        assert_eq!(
            parsed_file.longest_title,
            new_bookmark.title().chars().count()
        );
        assert_eq!(
            parsed_file.longest_category,
            new_bookmark.category().chars().count()
        )
    }

    #[test]
    fn test_parsed_file_remove_bookmark() {
        let mut plain_text = PlainText::new(PathBuf::from("test.txt"));
        let mut parsed_file = ParsedFile::new(plain_text.bookmarks());
        let bookmark = Bookmark::default();
        let char_count = bookmark.title().chars().count();
        let url = bookmark.url().to_string();
        parsed_file.add_bookmark(&mut plain_text, bookmark);
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

    #[test]
    fn test_parsed_file_add_char_count() {
        let mut char_count_vec = [0; 10];
        let mut longest: usize = 0;
        let field = "test";
        let char_count = field.chars().count();
        ParsedFile::add_char_count(&mut char_count_vec, &mut longest, field);
        assert_eq!(longest, char_count);
    }

    #[test]
    fn test_parsed_file_remove_char_count() {
        let mut char_count_vec = [0; 10];
        let mut longest: usize = 0;
        let field1 = "1";
        let char_count1 = field1.chars().count();
        let field2 = "22";
        let char_count2 = field2.chars().count();
        let field3 = "333";
        let char_count3 = field3.chars().count();
        let field4 = "4444";
        let char_count4 = field4.chars().count();
        ParsedFile::add_char_count(&mut char_count_vec, &mut longest, field1);
        ParsedFile::add_char_count(&mut char_count_vec, &mut longest, field2);
        ParsedFile::add_char_count(&mut char_count_vec, &mut longest, field3);
        ParsedFile::add_char_count(&mut char_count_vec, &mut longest, field4);
        assert_eq!(longest, char_count4);
        ParsedFile::remove_char_count(&mut char_count_vec, &mut longest, field4);
        assert_eq!(longest, char_count3);
        ParsedFile::remove_char_count(&mut char_count_vec, &mut longest, field3);
        assert_eq!(longest, char_count2);
        ParsedFile::remove_char_count(&mut char_count_vec, &mut longest, field2);
        assert_eq!(longest, char_count1);
        ParsedFile::remove_char_count(&mut char_count_vec, &mut longest, field1);
        assert_eq!(longest, 0);
    }
}
