use models::{CommentEntry, ContentEntry, DesktopFile, Entry, Group, GroupContent, TopLevelEntry};
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, take_while},
    character::complete::{char, line_ending, multispace1, space0},
    combinator::{eof, map, map_res, opt, value},
    error::Error,
    multi::{many0, many_till},
    sequence::{delimited, pair, preceded, terminated},
    AsChar, IResult, Parser,
};

pub mod models;

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

impl TryFrom<&str> for DesktopFile {
    type Error = nom::Err<Error<Vec<u8>>>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
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
    let (input, (header, content)) = pair(parse_group_header, parse_group_content).parse(input)?;

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
        terminated(
            delimited(char('['), take_while(|c| c != b'[' && c != b']'), char(']')),
            opt(char('\n')),
        ),
        str::from_utf8,
    )
    .parse(input)
}

fn parse_group_content(input: &[u8]) -> IResult<&[u8], GroupContent> {
    many0(parse_entry).parse(input)
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
            terminated(
                take_while(|c: u8| {
                    let item = c.as_char();
                    item.is_ascii() && !item.is_newline()
                }),
                char('\n'),
            ),
        ),
        str::from_utf8,
    )
    .parse(input)?;

    Ok((input, CommentEntry::Text(comment.to_owned())))
}

fn parse_key(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(
        take_while(|c: u8| {
            let item = c.as_char();
            item.is_alphanumeric() || item == '-'
        }),
        str::from_utf8,
    )
    .parse(input)
}

fn parse_value(input: &[u8]) -> IResult<&[u8], Vec<String>> {
    map(
        many_till(parse_single_value, alt((line_ending, eof))),
        |r| r.0,
    )
    .parse(input)
}

fn parse_single_value(input: &[u8]) -> IResult<&[u8], String> {
    terminated(
        map_res(
            escaped_transform(
                is_not("\\;\n"),
                '\\',
                map(
                    alt((
                        value("\\n", char('n')),
                        value("\\r", char('r')),
                        value("\\s", char('s')),
                        value("\\t", char('t')),
                        value("\\", char('\\')),
                        value("\\;", char(';')),
                    )),
                    |s| s.as_bytes(),
                ),
            ),
            |v| String::from_utf8(v).map(|s| s.trim().to_owned()),
        ),
        opt(char(';')),
    )
    .parse(input)
}

fn parse_content_entry(input: &[u8]) -> IResult<&[u8], ContentEntry> {
    let (input, key) = parse_key.parse(input)?;
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
    let (input, values) = parse_value.parse(input)?;
    Ok((
        input,
        ContentEntry {
            key: key.to_owned(),
            values,
            locale: locale.map(|tuple| tuple.1.to_owned()),
        },
    ))
}

#[cfg(test)]
mod tests {

    use nom::{error::ErrorKind, error_position};

    use super::{parse_entry, *};

    #[test]
    fn test_parse_entry() {
        let simple_entry = "Hello=World\n";
        let locale_entry = "Hello[locale]=World";
        let no_value = "Hello=";
        let comment = "# Comment\n";
        let empty = "\n";

        assert_eq!(
            parse_entry(simple_entry.as_bytes()),
            Ok((
                "".as_bytes(),
                Entry::Content(ContentEntry {
                    key: "Hello".to_owned(),
                    values: vec!["World".to_owned()],
                    locale: None
                })
            ))
        );

        assert_eq!(
            parse_entry(locale_entry.as_bytes()),
            Ok((
                "".as_bytes(),
                Entry::Content(ContentEntry {
                    key: "Hello".to_owned(),
                    values: vec!["World".to_owned()],
                    locale: Some("locale".to_owned())
                })
            ))
        );

        assert_eq!(
            parse_entry(comment.as_bytes()),
            Ok((
                "".as_bytes(),
                Entry::Comment(CommentEntry::Text("Comment".to_owned()))
            ))
        );

        assert_eq!(
            parse_entry(empty.as_bytes()),
            Ok((
                "".as_bytes(),
                Entry::Comment(CommentEntry::Blank("\n".to_owned()))
            ))
        );

        assert_eq!(
            parse_entry(no_value.as_bytes()),
            Ok((
                "".as_bytes(),
                Entry::Content(ContentEntry {
                    key: String::from("Hello"),
                    values: vec![],
                    locale: None
                })
            ))
        )
    }

    #[test]
    fn test_group_parsing() {
        let group = "[Desktop]
Type=Application
Exec=sh-test
Id=4
Hidden=false
";

        assert_eq!(
            parse_group(group.as_bytes()),
            Ok((
                "".as_bytes(),
                Group {
                    header: String::from("Desktop"),
                    content: vec![
                        Entry::Content(ContentEntry {
                            key: String::from("Type"),
                            values: vec![String::from("Application")],
                            locale: None
                        }),
                        Entry::Content(ContentEntry {
                            key: String::from("Exec"),
                            values: vec![String::from("sh-test")],
                            locale: None
                        }),
                        Entry::Content(ContentEntry {
                            key: String::from("Id"),
                            values: vec![String::from("4")],
                            locale: None
                        }),
                        Entry::Content(ContentEntry {
                            key: String::from("Hidden"),
                            values: vec![String::from("false")],
                            locale: None
                        }),
                    ],
                }
            ))
        );
    }

    #[test]
    fn test_bad_parsing() {
        let space_in_key = "Hello World=Yay";
        let bad_entry = "Hell[test]o=World";

        assert_eq!(
            parse_entry(space_in_key.as_bytes()),
            Err(nom::Err::Error(error_position!(
                "World=Yay".as_bytes(),
                ErrorKind::Char
            )))
        );

        assert_eq!(
            parse_entry(bad_entry.as_bytes()),
            Err(nom::Err::Error(error_position!(
                "o=World".as_bytes(),
                ErrorKind::Char
            )))
        );
    }

    #[test]
    fn test_multi_values_parsing() {
        let values_without_final_semi = "World;Universe;all others";
        let values = format!("{};", values_without_final_semi);

        assert_eq!(
            parse_value(values.as_bytes()),
            Ok((
                "".as_bytes(),
                vec![
                    String::from("World"),
                    String::from("Universe"),
                    String::from("all others")
                ]
            ))
        );
        assert_eq!(
            parse_value(values.as_bytes()),
            parse_value(values_without_final_semi.as_bytes())
        )
    }

    #[test]
    fn test_full_parsing() {
        let single = "# Outside comment
[Desktop]
Type=Application
Exec=sh test
Id=4
Hidden=false
";
        assert_eq!(
            DesktopFile::try_from(single),
            Ok(DesktopFile {
                content: vec![
                    TopLevelEntry::Comment(CommentEntry::Text("Outside comment".to_owned())),
                    TopLevelEntry::Group(Group {
                        header: "Desktop".to_owned(),
                        content: vec![
                            Entry::Content(ContentEntry {
                                key: String::from("Type"),
                                values: vec![String::from("Application")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("Exec"),
                                values: vec![String::from("sh test")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("Id"),
                                values: vec![String::from("4")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("Hidden"),
                                values: vec![String::from("false")],
                                locale: None
                            })
                        ]
                    })
                ],
            })
        );
    }
}
