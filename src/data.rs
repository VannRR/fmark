use std::path::PathBuf;

use crate::bookmark::Bookmark;
use crate::parsed_file::*;
use crate::plain_text::PlainText;

pub struct Data {
    plain_text: PlainText,
    parsed_file: ParsedFile,
}

impl Data {
    pub fn new(file_path: PathBuf) -> Result<Self, String> {
        let mut plain_text = PlainText::new(file_path)?;
        plain_text.read()?;
        let parsed_file = ParsedFile::new(plain_text.bookmarks());
        Ok(Self {
            plain_text,
            parsed_file,
        })
    }

    pub fn plain_text_bookmarks(&mut self) -> &str {
        self.plain_text.update_bookmarks(&self.parsed_file);
        self.plain_text.bookmarks()
    }

    pub fn plain_text_categories(&mut self) -> &str {
        self.plain_text.update_categories(&self.parsed_file);
        self.plain_text.categories()
    }

    pub fn write(&mut self) -> Result<(), String> {
        self.plain_text.write(&self.parsed_file)
    }

    pub fn set_bookmark(&mut self, new_bookmark: Bookmark, old_bookmark: Option<Bookmark>) {
        if let Some(old_bookmark) = old_bookmark {
            self.remove_bookmark(old_bookmark.url());
        }
        if self
            .parsed_file
            .add_category(new_bookmark.category().to_string())
        {
            self.plain_text.increment_categories_version();
        };
        self.parsed_file
            .update_longest_title(new_bookmark.title().to_string().chars().count());
        self.parsed_file
            .update_longest_category(new_bookmark.category().chars().count());
        self.parsed_file
            .bookmarks
            .insert(new_bookmark.url().to_string(), new_bookmark);
        self.plain_text.increment_bookmarks_version();
        self.plain_text.set_edited_true();
    }

    pub fn remove_bookmark(&mut self, url: &str) {
        if let Some(bookmark) = self.parsed_file.bookmarks.remove(url) {
            let category = bookmark.category();
            if self.parsed_file.remove_category(category) {
                self.plain_text.increment_categories_version();
            }
            self.parsed_file
                .revert_longest_title(bookmark.title().chars().count());
            self.parsed_file
                .revert_longest_category(bookmark.category().chars().count());
            self.plain_text.increment_bookmarks_version();
            self.plain_text.set_edited_true();
        }
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
    fn test_data_set_bookmark() {
        let mut data = Data::new(PathBuf::from("test.txt")).unwrap();
        let bookmark = Bookmark::default();
        data.set_bookmark(bookmark, None);
        assert_eq!(data.parsed_file.bookmarks.len(), 1);
        assert_eq!(data.parsed_file.categories().len(), 1);
        assert!(data.plain_text.edited());
    }

    #[test]
    fn test_data_remove_bookmark() {
        let mut data = Data::new(PathBuf::from("test.txt")).unwrap();
        let bookmark = Bookmark::default();
        let url = bookmark.url().to_string();
        data.set_bookmark(bookmark, None);
        data.remove_bookmark(&url);
        assert!(data.parsed_file.bookmarks.is_empty());
        assert!(data.parsed_file.categories().is_empty());
        assert!(data.plain_text.edited());
    }
}
