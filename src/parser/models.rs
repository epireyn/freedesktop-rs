use std::fmt::Display;

use crate::error::Error;

/// Trait implemented by entries to dynamically check whether the entry is blank or a comment.
pub trait CanBeComment {
    /// Returns whether the entry is a blank line.
    fn is_blank(&self) -> bool;
    /// Returns whether the entry is a comment.
    fn is_comment(&self) -> bool;
}

/// Trait implemented by entites that contain multiple entries (eg. [Group] and [TopLevelEntry]).
pub trait EntrySet<E> {
    /// Returns a vector of the entries that are not comments or blanks.
    fn without_comments(&self) -> Vec<&E>;

    /// Returs a vector of the entries that are comments or blanks (no key-values entries).
    fn only_comments(&self) -> Vec<&CommentEntry>;

    /// Find the first entry for this key, or `None` if no entry with this key was found.
    fn find(&self, key: &str) -> Option<&E>;

    /// Similar to [Self::find], but throws if the key is not found.
    fn get(&self, key: &str) -> Result<&E, Error> {
        self.find(key).ok_or(Error::NotFound(key.to_owned()))
    }

    /// Find the first entry for this key and returns it as a mutable reference, or `None` if no entry with this key was found.
    fn find_mut(&mut self, key: &str) -> Option<&mut E>;

    /// Similar to [Self::find_mut], but throws if the key is not found.    
    fn get_mut(&mut self, key: &str) -> Result<&mut E, Error> {
        self.find_mut(key).ok_or(Error::NotFound(key.to_owned()))
    }
}

/// Defines what options of a [Locale] are significant when searching for an entry.
pub struct LocaleOptions<'a> {
    /// A reference to the locale to be found.
    pub locale: &'a Locale,
    /// Whether the country is significant.
    pub country: bool,
    /// Whether the country is significant.
    pub encoding: bool,
    /// Whether the country is significant.
    pub modifier: bool,
}

impl<'a> LocaleOptions<'a> {
    /// Creates a new instance in which all options are insignificant (besides language).
    pub fn new(locale: &'a Locale) -> Self {
        Self {
            locale,
            country: false,
            encoding: false,
            modifier: false,
        }
    }
    /// Creates a new instance in which all options are significant.
    pub fn all(locale: &'a Locale) -> Self {
        Self {
            locale,
            country: true,
            encoding: true,
            modifier: true,
        }
    }

    /// Makes all options insignificant but the language.
    pub fn language_only(self) -> Self {
        Self {
            locale: self.locale,
            country: false,
            encoding: false,
            modifier: false,
        }
    }

    /// Makes all options significant.
    pub fn all_significant(self) -> Self {
        self.significant_country()
            .significant_encoding()
            .significant_modifiers()
    }

    /// Makes the encoding significant.
    pub fn significant_encoding(mut self) -> Self {
        self.encoding = true;
        self
    }

    /// Makes the country significant.
    pub fn significant_country(mut self) -> Self {
        self.country = true;
        self
    }

    /// Makes the modifiers significant.
    pub fn significant_modifiers(mut self) -> Self {
        self.modifier = true;
        self
    }

    /// Changes the locale.
    pub fn with_locale(mut self, locale: &'a Locale) -> Self {
        self.locale = locale;
        self
    }
}

/// A group of entries.
///
/// This represents a section in a freedesktop file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Group {
    /// The section name.
    pub header: String,

    /// The content of the section.
    pub content: GroupContent,
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}]", self.header)?;

        write_content(f, &self.content)
    }
}

impl EntrySet<ContentEntry> for Group {
    fn without_comments(&self) -> Vec<&ContentEntry> {
        self.content
            .iter()
            .filter_map(|e| {
                if let Entry::Content(content) = e {
                    Some(content)
                } else {
                    None
                }
            })
            .collect()
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

    fn find(&self, key: &str) -> Option<&ContentEntry> {
        self.content
            .iter()
            .filter_map(|i| match i {
                Entry::Content(content_entry) => Some(content_entry),
                Entry::Comment(_) => None,
            })
            .find(|e| e.key == key)
    }

    fn find_mut(&mut self, key: &str) -> Option<&mut ContentEntry> {
        self.content
            .iter_mut()
            .filter_map(|i| match i {
                Entry::Content(content_entry) => Some(content_entry),
                Entry::Comment(_) => None,
            })
            .find(|e| e.key == key)
    }
}

impl Group {
    /// Find the first entry for this key and locale, or `None` if no entry with this key was found.
    pub fn find_with_locale(&self, key: &str, options: &LocaleOptions) -> Option<&ContentEntry> {
        self.content
            .iter()
            .filter_map(|i| match i {
                Entry::Content(content_entry) => Some(content_entry),
                Entry::Comment(_) => None,
            })
            .find(|e| {
                e.key == key
                    && e.locale
                        .as_ref()
                        .map_or(true, |locale| locale.equals_options(options))
            })
    }
    /// Find the first entry for this key and locale and returns it as a mutable reference, or `None` if no entry with this key was found.
    pub fn find_with_locale_mut(
        &mut self,
        key: &str,
        options: &LocaleOptions,
    ) -> Option<&mut ContentEntry> {
        self.content
            .iter_mut()
            .filter_map(|i| match i {
                Entry::Content(content_entry) => Some(content_entry),
                Entry::Comment(_) => None,
            })
            .find(|e| {
                e.key == key
                    && e.locale
                        .as_ref()
                        .map_or(true, |locale| locale.equals_options(options))
            })
    }
}

/// Content of a section.
///
/// This an alias for Vec<Entry>.
pub type GroupContent = Vec<Entry>;

/// An entry in the file
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Entry {
    /// A key-values entry
    Content(ContentEntry),
    /// A comment or blank line
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

/// An entry at the root of the file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TopLevelEntry {
    /// A section as per the Freedesktop specification.
    Group(Group),

    /// A comment or blank line.
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

/// A comment or a blank line.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CommentEntry {
    /// A textual comment. Contains the line content.
    Text(String),
    /// A blank line. Containes a number of "\n" for each subsequent lines.
    Blank(String),
}

impl Display for CommentEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommentEntry::Text(s) => write!(f, "# {s}"),

            // No line feed since it's in its content already
            CommentEntry::Blank(s) => write!(f, "{s}"),
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

/// A key-values entry.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ContentEntry {
    /// The key of the entry.
    pub key: String,

    /// The values of the entry.
    pub values: Vec<String>,

    /// The potential locale of the entry.
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

/// A locale of an entry.
///
/// If given to an entry, the only required argument is the language. Everything else is optional.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Locale {
    /// The language of the bound value.
    pub lang: String,
    /// The encoding of the bound value.
    pub encoding: Option<String>,
    /// The country variant of the language as per the specification.
    pub country: Option<String>,
    /// Any other variant of the language as per the specification.
    pub modifiers: Option<String>,
}

impl Locale {
    /// Check whether this locale respects the options
    pub fn equals_options(&self, options: &LocaleOptions) -> bool {
        let rhs = options.locale;
        let mut res = self.lang == rhs.lang;
        if options.country {
            res &= self.country == rhs.country;
        }
        if options.encoding {
            res &= self.encoding == rhs.encoding;
        }
        if options.modifier {
            res &= self.modifiers == rhs.modifiers;
        }
        res
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lang)?;
        if let Some(country) = &self.country {
            write!(f, "_{}", country.to_uppercase())?;
        }

        if let Some(encoding) = &self.encoding {
            write!(f, ".{encoding}")?;
        }

        if let Some(modifier) = &self.modifiers {
            write!(f, "@{modifier}")?;
        }

        Ok(())
    }
}

/// The representation of a Freedesktop file, which contains [TopLevelEntry].
///
/// This struct is used to parse raw data, see its implementations of [From<...>] for more information.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DesktopFile {
    /// The top-level entries of the file.
    pub content: Vec<TopLevelEntry>,
}

impl EntrySet<Group> for DesktopFile {
    fn without_comments(&self) -> Vec<&Group> {
        self.content
            .iter()
            .filter_map(|tle| match tle {
                TopLevelEntry::Group(group) => Some(group),
                _ => None,
            })
            .collect()
    }

    fn only_comments(&self) -> Vec<&CommentEntry> {
        self.content
            .iter()
            .filter_map(|tle| match tle {
                TopLevelEntry::Comment(comment) => Some(comment),
                _ => None,
            })
            .collect()
    }

    fn find(&self, header: &str) -> Option<&Group> {
        self.without_comments()
            .iter()
            .find(|g| g.header == header)
            .copied()
    }

    fn find_mut(&mut self, header: &str) -> Option<&mut Group> {
        self.content
            .iter_mut()
            .filter_map(|i| match i {
                TopLevelEntry::Group(group) => Some(group),
                _ => None,
            })
            .find(|g| g.header == header)
    }
}

impl Display for DesktopFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_content(f, &self.content)
    }
}

fn write_content<T: CanBeComment + Display>(
    f: &mut std::fmt::Formatter<'_>,
    content: &[T],
) -> std::fmt::Result {
    let mut peekable = content.iter().peekable();
    while let Some(item) = peekable.next() {
        item.fmt(f)?;

        // Add new line if it is a written entry before the end of iteration
        if peekable.peek().is_some() && !item.is_blank() {
            writeln!(f)?;
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
                modifiers: Some(String::from("new")),
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
