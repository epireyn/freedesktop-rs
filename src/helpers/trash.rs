use time::{format_description::BorrowedFormatItem, macros::format_description, PrimitiveDateTime};

use crate::parser::models::{ContentEntry, DesktopFile, Entry, EntrySet, Group, TopLevelEntry};

const DATE_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day]T[hour repr:24]:[minute]:[second]");

const GROUP_NAME: &str = "Trash Info";

/// Representation of a freedesktop trash file.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TrashFile {
    desktop_file: DesktopFile,
    /// Path of the trashed file.
    pub path: String,
    /// Deletion date of the trashed file.
    pub deletion_date: PrimitiveDateTime,
}

impl TryFrom<TrashFile> for DesktopFile {
    type Error = crate::error::Error;

    fn try_from(trash_file: TrashFile) -> Result<Self, Self::Error> {
        let mut desktop_file = trash_file.desktop_file;
        let group = desktop_file.find_mut(GROUP_NAME);

        let raw_date = trash_file
            .deletion_date
            .format(&DATE_FORMAT)
            .map_err(crate::error::Error::DateFormat)?;

        let new_deletion_date = Entry::Content(ContentEntry {
            key: String::from("DeletionDate"),
            values: vec![raw_date.clone()],
            locale: None,
        });

        let new_path = Entry::Content(ContentEntry {
            key: String::from("Path"),
            values: vec![trash_file.path.clone()],
            locale: None,
        });

        if let Some(group) = group {
            let path = group.find_mut("Path");
            if let Some(path) = path {
                path.values = vec![trash_file.path];
            } else {
                group.content.push(new_path);
            }

            let date = group.find_mut("DeletionDate");
            if let Some(date) = date {
                date.values = vec![raw_date];
            } else {
                group.content.push(new_deletion_date);
            }
        } else {
            let group = Group {
                header: String::from(GROUP_NAME),
                content: vec![new_path, new_deletion_date],
            };
            desktop_file.content.push(TopLevelEntry::Group(group));
        }

        Ok(desktop_file)
    }
}

impl TryFrom<DesktopFile> for TrashFile {
    type Error = crate::error::Error;

    fn try_from(desktop: DesktopFile) -> Result<Self, Self::Error> {
        let group = desktop.get(GROUP_NAME)?;
        let raw_date = group.get("DeletionDate")?;
        let raw_path = group.get("Path")?;

        let date = PrimitiveDateTime::parse(&raw_date.values[0], &DATE_FORMAT)
            .map_err(crate::error::Error::DateParsing)?;

        let path = raw_path.values[0].to_owned();

        Ok(Self {
            desktop_file: desktop,
            path,
            deletion_date: date,
        })
    }
}

#[cfg(test)]
mod test {
    use time::macros::datetime;

    use crate::parser::models::CommentEntry;

    use super::*;

    #[test]
    fn parse_proper_file() {
        let trash_file = "[Trash Info]
Path=~/Downloads/file
DeletionDate=2025-08-12T00:14:20";

        assert_eq!(
            TrashFile::try_from(DesktopFile::try_from(trash_file).unwrap()).unwrap(),
            TrashFile {
                desktop_file: DesktopFile {
                    content: vec![TopLevelEntry::Group(Group {
                        header: String::from("Trash Info"),
                        content: vec![
                            Entry::Content(ContentEntry {
                                key: String::from("Path"),
                                values: vec![String::from("~/Downloads/file")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("DeletionDate"),
                                values: vec![String::from("2025-08-12T00:14:20")],
                                locale: None
                            })
                        ],
                    })],
                },
                path: String::from("~/Downloads/file"),
                deletion_date: datetime!(2025-08-12 00:14:20)
            }
        )
    }
    #[test]
    fn parse_double_entry_file() {
        let trash_file = "[Trash Info]
DeletionDate=2025-08-12T00:14:20
Path=~/Downloads/file
Path=/wrong/
DeletionDate=2025-08-14T00:00:00
";

        assert_eq!(
            TrashFile::try_from(DesktopFile::try_from(trash_file).unwrap()).unwrap(),
            TrashFile {
                desktop_file: DesktopFile {
                    content: vec![TopLevelEntry::Group(Group {
                        header: String::from("Trash Info"),
                        content: vec![
                            Entry::Content(ContentEntry {
                                key: String::from("DeletionDate"),
                                values: vec![String::from("2025-08-12T00:14:20")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("Path"),
                                values: vec![String::from("~/Downloads/file")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("Path"),
                                values: vec![String::from("/wrong/")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("DeletionDate"),
                                values: vec![String::from("2025-08-14T00:00:00")],
                                locale: None
                            })
                        ],
                    })],
                },
                path: String::from("~/Downloads/file"),
                deletion_date: datetime!(2025-08-12 00:14:20)
            }
        )
    }

    #[test]
    fn edit_and_convert_file() {
        let mut trash_file = TrashFile {
            desktop_file: DesktopFile {
                content: vec![TopLevelEntry::Group(Group {
                    header: String::from("Trash Info"),
                    content: vec![
                        Entry::Content(ContentEntry {
                            key: String::from("DeletionDate"),
                            values: vec![String::from("2025-08-12T00:14:20")],
                            locale: None,
                        }),
                        Entry::Content(ContentEntry {
                            key: String::from("Path"),
                            values: vec![String::from("~/Downloads/file")],
                            locale: None,
                        }),
                        Entry::Content(ContentEntry {
                            key: String::from("Path"),
                            values: vec![String::from("/wrong/")],
                            locale: None,
                        }),
                        Entry::Content(ContentEntry {
                            key: String::from("DeletionDate"),
                            values: vec![String::from("2025-08-14T00:00:00")],
                            locale: None,
                        }),
                    ],
                })],
            },
            path: String::from("~/Downloads/file"),
            deletion_date: datetime!(2025-08-12 00:14:20),
        };

        assert_eq!(
            DesktopFile::try_from(trash_file.clone())
                .unwrap()
                .to_string(),
            "[Trash Info]
DeletionDate=2025-08-12T00:14:20
Path=~/Downloads/file
Path=/wrong/
DeletionDate=2025-08-14T00:00:00"
        );

        trash_file.path = String::from("/new/path");

        assert_eq!(
            DesktopFile::try_from(trash_file.clone())
                .unwrap()
                .to_string(),
            "[Trash Info]
DeletionDate=2025-08-12T00:14:20
Path=/new/path
Path=/wrong/
DeletionDate=2025-08-14T00:00:00"
        );
    }

    #[test]
    fn preserve_comments() {
        let trash_file = "[Trash Info]


DeletionDate=2025-08-12T00:14:20
# Here is an awesome comment
Path=~/Downloads/file
Path=/wrong/


DeletionDate=2025-08-14T00:00:00";
        let trash_object = TrashFile::try_from(DesktopFile::try_from(trash_file).unwrap()).unwrap();

        assert_eq!(
            trash_object,
            TrashFile {
                desktop_file: DesktopFile {
                    content: vec![TopLevelEntry::Group(Group {
                        header: String::from("Trash Info"),
                        content: vec![
                            Entry::Comment(CommentEntry::Blank(String::from("\n\n"))),
                            Entry::Content(ContentEntry {
                                key: String::from("DeletionDate"),
                                values: vec![String::from("2025-08-12T00:14:20")],
                                locale: None
                            }),
                            Entry::Comment(CommentEntry::Text(String::from(
                                "Here is an awesome comment"
                            ))),
                            Entry::Content(ContentEntry {
                                key: String::from("Path"),
                                values: vec![String::from("~/Downloads/file")],
                                locale: None
                            }),
                            Entry::Content(ContentEntry {
                                key: String::from("Path"),
                                values: vec![String::from("/wrong/")],
                                locale: None
                            }),
                            Entry::Comment(CommentEntry::Blank(String::from("\n\n"))),
                            Entry::Content(ContentEntry {
                                key: String::from("DeletionDate"),
                                values: vec![String::from("2025-08-14T00:00:00")],
                                locale: None
                            })
                        ],
                    })],
                },
                path: String::from("~/Downloads/file"),
                deletion_date: datetime!(2025-08-12 00:14:20)
            }
        );

        assert_eq!(
            DesktopFile::try_from(trash_object).unwrap().to_string(),
            trash_file
        );
    }
}
