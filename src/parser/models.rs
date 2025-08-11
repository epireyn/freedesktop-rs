use std::fmt::Display;

trait CanBeComment {
    fn is_blank(&self) -> bool;
    fn is_comment(&self) -> bool;
}

pub trait EntrySet<E> {
    fn without_comments(&self) -> Vec<&E>;
    fn only_comments(&self) -> Vec<&CommentEntry>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct Group {
    pub header: String,
    pub content: GroupContent,
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]\n", self.header)?;

        write_content(f, &self.content)
    }
}

impl EntrySet<Entry> for Group {
    fn without_comments(&self) -> Vec<&Entry> {
        self.content.iter().filter(|e| !e.is_comment()).collect()
    }

    fn only_comments(&self) -> Vec<&CommentEntry> {
        self.content
            .iter()
            .filter_map(|e| {
                if let Entry::Comment(comment) = e {
                    Some(comment)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub type GroupContent = Vec<Entry>;

#[derive(Debug, Eq, PartialEq)]
pub enum Entry {
    Content(ContentEntry),
    Comment(CommentEntry),
}

impl From<CommentEntry> for Entry {
    fn from(value: CommentEntry) -> Self {
        Entry::Comment(value)
    }
}

impl From<ContentEntry> for Entry {
    fn from(value: ContentEntry) -> Self {
        Entry::Content(value)
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Content(content_entry) => content_entry.fmt(f),
            Entry::Comment(comment_entry) => comment_entry.fmt(f),
        }
    }
}

impl CanBeComment for Entry {
    fn is_comment(&self) -> bool {
        match self {
            Entry::Content(_) => false,
            Entry::Comment(_) => true,
        }
    }

    fn is_blank(&self) -> bool {
        if let Self::Comment(comment) = self {
            comment.is_blank()
        } else {
            false
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum TopLevelEntry {
    Group(Group),
    Comment(CommentEntry),
}

impl From<CommentEntry> for TopLevelEntry {
    fn from(value: CommentEntry) -> Self {
        TopLevelEntry::Comment(value)
    }
}

impl From<Group> for TopLevelEntry {
    fn from(value: Group) -> Self {
        TopLevelEntry::Group(value)
    }
}

impl Display for TopLevelEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopLevelEntry::Group(group) => group.fmt(f),
            TopLevelEntry::Comment(comment_entry) => comment_entry.fmt(f),
        }
    }
}

impl CanBeComment for TopLevelEntry {
    fn is_comment(&self) -> bool {
        match self {
            TopLevelEntry::Group(_) => false,
            TopLevelEntry::Comment(_) => true,
        }
    }

    fn is_blank(&self) -> bool {
        if let Self::Comment(comment) = self {
            comment.is_blank()
        } else {
            false
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CommentEntry {
    Text(String),
    Blank(String),
}

impl Display for CommentEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentEntry::Text(s) => write!(f, "# {}", s),

            // No line feed since it's in its content already
            CommentEntry::Blank(s) => write!(f, "{}", s),
        }
    }
}

impl CommentEntry {
    fn is_blank(&self) -> bool {
        match self {
            CommentEntry::Text(_) => false,
            CommentEntry::Blank(_) => true,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContentEntry {
    pub key: String,
    pub values: Vec<String>,
    pub locale: Option<Locale>,
}

impl Display for ContentEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)?;
        if let Some(locale) = &self.locale {
            write!(f, "[")?;
            locale.fmt(f)?;
            write!(f, "]")?;
        }
        write!(f, "={}", self.values.join(";"))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Locale {
    pub lang: String,
    pub encoding: Option<String>,
    pub country: Option<String>,
    pub modifier: Option<String>,
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lang)?;
        if let Some(country) = &self.country {
            write!(f, "_{}", country.to_uppercase())?;
        }

        if let Some(encoding) = &self.encoding {
            write!(f, ".{}", encoding)?;
        }

        if let Some(modifier) = &self.modifier {
            write!(f, "@{}", modifier)?;
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DesktopFile {
    pub content: Vec<TopLevelEntry>,
}

impl Display for DesktopFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_content(f, &self.content)
    }
}

fn write_content<T: CanBeComment + Display>(
    f: &mut std::fmt::Formatter<'_>,
    content: &Vec<T>,
) -> std::fmt::Result {
    let mut peekable = content.iter().peekable();
    while let Some(item) = peekable.next() {
        item.fmt(f)?;

        // Add new line if it is a written entry before the end of iteration
        if peekable.peek().is_some() && !item.is_blank() {
            write!(f, "\n")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_value_entry_format() {
        let single_value_entry = Entry::Content(ContentEntry {
            key: String::from("Hello"),
            values: vec![String::from("World")],
            locale: None,
        });
        let single_value_locale_entry = Entry::Content(ContentEntry {
            key: String::from("Hello"),
            values: vec![String::from("World")],
            locale: Some(Locale {
                lang: String::from("en"),
                encoding: None,
                country: Some(String::from("US")),
                modifier: Some(String::from("new")),
            }),
        });

        assert_eq!(&single_value_entry.to_string(), "Hello=World");
        assert_eq!(
            &single_value_locale_entry.to_string(),
            "Hello[en_US@new]=World"
        );
    }

    #[test]
    fn test_multi_values_entry_format() {
        let multi_values = Entry::Content(ContentEntry {
            key: String::from("Hello"),
            values: vec![
                String::from("World"),
                String::from(" Universe"),
                String::from("all others"),
            ],
            locale: None,
        });
        assert_eq!(
            &multi_values.to_string(),
            "Hello=World; Universe;all others"
        );
    }

    #[test]
    fn test_comments_format() {
        let text_comment = Entry::Comment(CommentEntry::Text(String::from("Test with spaces")));
        let blank_comment = Entry::Comment(CommentEntry::Blank(String::from("\n\t")));

        assert_eq!(&text_comment.to_string(), "# Test with spaces");
        assert_eq!(&blank_comment.to_string(), "\n\t");
    }

    #[test]
    fn test_full_file() {
        let file = DesktopFile {
            content: vec![
                TopLevelEntry::Comment(CommentEntry::Text(String::from("First group"))),
                TopLevelEntry::Group(Group {
                    header: String::from("First"),
                    content: vec![
                        Entry::Content(ContentEntry {
                            key: String::from("Title"),
                            values: vec![String::from("First group")],
                            locale: None,
                        }),
                        Entry::Comment(CommentEntry::Blank(String::from("\n"))),
                        Entry::Comment(CommentEntry::Text(String::from("End of group"))),
                    ],
                }),
                TopLevelEntry::Group(Group {
                    header: String::from("Second"),
                    content: vec![],
                }),
            ],
        };

        assert_eq!(
            &file.to_string(),
            "# First group
[First]
Title=First group

# End of group
[Second]
"
        )
    }
}
