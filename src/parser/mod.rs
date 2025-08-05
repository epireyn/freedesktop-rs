use nom::{
    branch::alt,
    bytes::streaming::take_while,
    character::{
        char,
        complete::{alphanumeric0, alphanumeric1, multispace0, multispace1, space0},
    },
    combinator::map_res,
    error::Error,
    multi::{many, many0},
    sequence::{delimited, pair, preceded, terminated},
    IResult, Parser,
};

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
    pub value: String,
    pub locale: Option<String>,
}

#[derive(Debug)]
pub struct DesktopFile {
    pub content: Vec<TopLevelEntry>,
}

impl TryFrom<&[u8]> for DesktopFile {
    type Error = nom::Err<Error<Vec<u8>>>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let res = many0(parse_top_level_entry).parse(value);

        match res {
            Ok((_, content)) => Ok(Self { content }),
            Err(e) => Err(e.to_owned()),
        }
    }
}

fn parse_top_level_entry(input: &[u8]) -> IResult<&[u8], TopLevelEntry> {
    alt((
        map_res(parse_group, TopLevelEntry::try_from),
        map_res(parse_comment_entry, TopLevelEntry::try_from),
    ))
    .parse(input)
}

fn parse_group(input: &[u8]) -> IResult<&[u8], Group> {
    let (input, (header, content)) = (parse_group_header, parse_group_content).parse(input)?;

    Ok((
        input,
        Group {
            header: header.to_owned(),
            content,
        },
    ))
}

fn parse_group_header(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(
        delimited(char('['), take_while(|c| c != b'[' && c != b']'), char(']')),
        str::from_utf8,
    )
    .parse(input)
}

fn parse_group_content(input: &[u8]) -> IResult<&[u8], GroupContent> {
    many(0.., terminated(parse_entry, multispace0)).parse(input)
}

fn parse_entry(input: &[u8]) -> IResult<&[u8], Entry> {
    alt((
        map_res(parse_comment_entry, Entry::try_from),
        map_res(parse_content_entry, Entry::try_from),
    ))
    .parse(input)
}

fn parse_blank_comment_entry(input: &[u8]) -> IResult<&[u8], CommentEntry> {
    let (input, space) = map_res(multispace1, str::from_utf8).parse(input)?;

    Ok((input, CommentEntry::Blank(space.to_owned())))
}

fn parse_comment_entry(input: &[u8]) -> IResult<&[u8], CommentEntry> {
    alt((parse_blank_comment_entry, parse_text_comment_entry)).parse(input)
}

fn parse_text_comment_entry(input: &[u8]) -> IResult<&[u8], CommentEntry> {
    let (input, comment) = map_res(
        preceded(
            pair(char('#'), space0),
            terminated(alphanumeric1, char('\n')),
        ),
        str::from_utf8,
    )
    .parse(input)?;

    Ok((input, CommentEntry::Text(comment.to_owned())))
}

fn parse_content_entry(input: &[u8]) -> IResult<&[u8], ContentEntry> {
    let (input, key) = map_res(alphanumeric0, str::from_utf8).parse(input)?;
    let locale_result: IResult<&[u8], &str> = map_res(
        delimited(char('['), take_while(|c| c != b'[' && c != b']'), char(']')),
        |res| str::from_utf8(res),
    )
    .parse(input);

    let locale = locale_result.ok();

    let input = if let Some(input) = locale {
        input.0
    } else {
        input
    };

    let (input, _) = (space0, char('='), space0).parse(input)?;
    let (input, value) =
        map_res(terminated(alphanumeric1, char('\n')), str::from_utf8).parse(input)?;
    Ok((
        input,
        ContentEntry {
            key: key.to_owned(),
            value: value.to_owned(),
            locale: locale.map(|tuple| tuple.1.to_owned()),
        },
    ))
}
