#[derive(Debug, Eq, PartialEq)]
pub struct Group {
    pub header: String,
    pub content: GroupContent,
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

#[derive(Debug, Eq, PartialEq)]
pub enum CommentEntry {
    Text(String),
    Blank(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct ContentEntry {
    pub key: String,
    pub values: Vec<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DesktopFile {
    pub content: Vec<TopLevelEntry>,
}
