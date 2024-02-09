use std::borrow::Cow;

const TITLE_MARKER: &str = "T";
const TITLE_MAX_LENGTH: usize = 35;

const CATEGORY_MARKER: &str = "C";
const CATEGORY_MAX_LENGTH: usize = 35;

const URL_MARKER: &str = "U";
const URL_MAX_LENGTH: usize = 2048;

const SEGMENT_START: char = '{';
const SEGMENT_END: char = '}';

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
        let title = "Project's Github".to_string();
        let category = "Development".to_string();
        let url = "https://github.com/vannrr/fmark".to_string();
        Self {
            title,
            category,
            url,
        }
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn to_line(&self, mut title_padding: usize, mut category_padding: usize) -> String {
        let title_char_count = self.title().chars().count();
        if title_padding >= TITLE_MAX_LENGTH - 1 {
            title_padding = TITLE_MAX_LENGTH.saturating_sub(title_char_count) + 1
        } else {
            title_padding = title_padding.saturating_sub(title_char_count) + 1
        }

        let category_char_count = self.category().chars().count();
        if category_padding >= CATEGORY_MAX_LENGTH - 1 {
            category_padding = CATEGORY_MAX_LENGTH.saturating_sub(category_char_count) + 1
        } else {
            category_padding = category_padding.saturating_sub(category_char_count) + 1
        }

        let title: Cow<str> = if title_char_count > TITLE_MAX_LENGTH {
            format!("{:.TITLE_MAX_LENGTH$}", self.title).into()
        } else {
            Cow::Borrowed(&self.title)
        };

        let category: Cow<str> = if category_char_count > CATEGORY_MAX_LENGTH {
            format!("{:.CATEGORY_MAX_LENGTH$}", self.category).into()
        } else {
            Cow::Borrowed(&self.category)
        };

        let url: Cow<str> = if self.url.len() > URL_MAX_LENGTH {
            format!("{:.URL_MAX_LENGTH$}", self.url).into()
        } else {
            Cow::Borrowed(&self.url)
        };

        format!(
            "{{{}}}{{{}}}{:title_padding$}{{{}}}{{{}}}{:category_padding$}{{{}}}{{{}}}\n",
            TITLE_MARKER, title, "", CATEGORY_MARKER, category, "", URL_MARKER, url
        )
    }

    pub fn from_line(line: &str) -> Option<Bookmark> {
        let mut segments: Vec<String> = Vec::new();
        let mut segment = String::new();
        let mut capture = false;
        for c in line.chars() {
            if c == SEGMENT_START && !capture {
                capture = true;
                segment.clear();
            } else if c == '\n' && capture {
                capture = false;
                segment.clear();
            }
            if capture {
                segment.push(c);
            }
            if c == SEGMENT_END && capture {
                capture = false;
                if !segment.is_empty() {
                    segments.push(segment.clone());
                }
            }
        }
        if segments.len() == 6 {
            let mut title = None;
            let mut category = None;
            let mut url = None;
            for i in (0..segments.len()).step_by(2) {
                let marker = segments[i]
                    .trim_matches(&[SEGMENT_START, SEGMENT_END] as &[char])
                    .trim();

                let field = segments[i + 1]
                    .trim_matches(&[SEGMENT_START, SEGMENT_END] as &[char])
                    .trim();
                match marker {
                    TITLE_MARKER => title = Some(field),
                    CATEGORY_MARKER => category = Some(field),
                    URL_MARKER => url = Some(field),
                    _ => {}
                }
            }
            if let (Some(title), Some(category), Some(url)) = (title, category, url) {
                return Some(Bookmark::new(
                    title.to_string(),
                    category.to_string(),
                    url.to_string(),
                ));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_default() {
        let bookmark = Bookmark::default();
        assert_eq!(bookmark.title(), "Project's Github");
        assert_eq!(bookmark.category(), "Development");
        assert_eq!(bookmark.url(), "https://github.com/vannrr/fmark");
    }

    #[test]
    fn test_bookmark_formatted_line() {
        let bookmark = Bookmark::new(
            "Rust Programming".to_string(),
            "Programming".to_string(),
            "https://www.rust-lang.org/".to_string(),
        );

        let formatted_line = bookmark.to_line(25, 25);

        assert_eq!(
            formatted_line,
            format!(
                "{{{}}}{{{}}}{:10}{{{}}}{{{}}}{:15}{{{}}}{{{}}}\n",
                TITLE_MARKER,
                "Rust Programming",
                "",
                CATEGORY_MARKER,
                "Programming",
                "",
                URL_MARKER,
                "https://www.rust-lang.org/"
            )
        );
    }

    #[test]
    fn test_bookmark_from_line() {
        let default_bookmark = Bookmark::default();
        let bookmark = Bookmark::from_line(&default_bookmark.to_line(10, 10)).unwrap();
        assert_eq!(bookmark.title(), default_bookmark.title());
        assert_eq!(bookmark.category(), default_bookmark.category());
        assert_eq!(bookmark.url(), default_bookmark.url());
    }
}
