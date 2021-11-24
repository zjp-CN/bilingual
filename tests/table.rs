use bilingual::md::Md;
use insta::{assert_debug_snapshot, assert_display_snapshot};
use pulldown_cmark::{Event, Parser};

const TABLE: &str = "
| Option | Description |
| ------:| -----------:|
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |
";

#[test]
fn bilingual_test() {
    let mut md = Md::new(TABLE);
    let text = md.extract().to_owned();
    assert_debug_snapshot!(text, @r###""Option\nDescription\ndata\npath to data files to supply the data that will be passed into templates.\nengine\nengine to be used for processing templates. Handlebars is the default.\next\nextension to be used for dest files.\n""###);
    md.bytes_paragraph(0).last();
    assert_eq!(text, md.paragraphs());
    md.chars_paragraph(0).last();
    assert_eq!(text, md.paragraphs());
    md.bytes_paragraph(1000).last();
    assert_eq!(text, md.paragraphs());
    assert_debug_snapshot!(md, @r###"
    Md {
        events: [
            Start(
                Table(
                    [
                        Right,
                        Right,
                    ],
                ),
            ),
            Start(
                TableHead,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "Option",
                ),
            ),
            End(
                TableCell,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "Description",
                ),
            ),
            End(
                TableCell,
            ),
            End(
                TableHead,
            ),
            Start(
                TableRow,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "data",
                ),
            ),
            End(
                TableCell,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "path to data files to supply the data that will be passed into templates.",
                ),
            ),
            End(
                TableCell,
            ),
            End(
                TableRow,
            ),
            Start(
                TableRow,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "engine",
                ),
            ),
            End(
                TableCell,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "engine to be used for processing templates. Handlebars is the default.",
                ),
            ),
            End(
                TableCell,
            ),
            End(
                TableRow,
            ),
            Start(
                TableRow,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "ext",
                ),
            ),
            End(
                TableCell,
            ),
            Start(
                TableCell,
            ),
            Text(
                Borrowed(
                    "extension to be used for dest files.",
                ),
            ),
            End(
                TableCell,
            ),
            End(
                TableRow,
            ),
            End(
                Table(
                    [
                        Right,
                        Right,
                    ],
                ),
            ),
        ],
        buffer: "Option\nDescription\ndata\npath to data files to supply the data that will be passed into templates.\nengine\nengine to be used for processing templates. Handlebars is the default.\next\nextension to be used for dest files.\n",
        bytes: [
            7,
            12,
            5,
            74,
            7,
            71,
            4,
            37,
        ],
        chars: [
            7,
            12,
            5,
            74,
            7,
            71,
            4,
            37,
        ],
        limit: Limit {
            limit: 1000,
            cnt: 0,
            len: 217,
            pos: 0,
        },
    }
    "###);
    md.chars_paragraph(1000).last();
    assert_eq!(text, md.paragraphs());
}

#[test]
fn table_test() {
    let events = Parser::new_ext(TABLE, bilingual::md::cmark_opt()).collect::<Vec<_>>();
    assert_debug_snapshot!(events, @r###"
    [
        Start(
            Table(
                [
                    Right,
                    Right,
                ],
            ),
        ),
        Start(
            TableHead,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "Option",
            ),
        ),
        End(
            TableCell,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "Description",
            ),
        ),
        End(
            TableCell,
        ),
        End(
            TableHead,
        ),
        Start(
            TableRow,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "data",
            ),
        ),
        End(
            TableCell,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "path to data files to supply the data that will be passed into templates.",
            ),
        ),
        End(
            TableCell,
        ),
        End(
            TableRow,
        ),
        Start(
            TableRow,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "engine",
            ),
        ),
        End(
            TableCell,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "engine to be used for processing templates. Handlebars is the default.",
            ),
        ),
        End(
            TableCell,
        ),
        End(
            TableRow,
        ),
        Start(
            TableRow,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "ext",
            ),
        ),
        End(
            TableCell,
        ),
        Start(
            TableCell,
        ),
        Text(
            Borrowed(
                "extension to be used for dest files.",
            ),
        ),
        End(
            TableCell,
        ),
        End(
            TableRow,
        ),
        End(
            Table(
                [
                    Right,
                    Right,
                ],
            ),
        ),
    ]
    "###);
    assert_debug_snapshot!(
    events.iter().filter_map(|e| match e
    { 
        Event::Text(x) => Some(x.as_ref()),
        _ => None
    }).collect::<Vec<_>>(), @r###"
    [
        "Option",
        "Description",
        "data",
        "path to data files to supply the data that will be passed into templates.",
        "engine",
        "engine to be used for processing templates. Handlebars is the default.",
        "ext",
        "extension to be used for dest files.",
    ]
    "###);

    let paragraphs = events.iter()
                           .filter_map(|e| match e {
                               Event::Text(x) => Some(x.as_ref()),
                               _ => None,
                           })
                           .collect::<Vec<_>>()
                           .join("\n");
    assert_debug_snapshot!(paragraphs, @r###""Option\nDescription\ndata\npath to data files to supply the data that will be passed into templates.\nengine\nengine to be used for processing templates. Handlebars is the default.\next\nextension to be used for dest files.""###);
    assert_eq!(paragraphs, Md::new(TABLE).extract().trim());

    let translated = &mut paragraphs.split("\n");
    let events = events.into_iter()
                       .map(|e| match e {
                           t @ Event::Text(_) => {
                               vec![t,
                                    Event::Text('\t'.into()),
                                    Event::Text(translated.next().unwrap().into())]
                           }
                           x => vec![x],
                       })
                       .flatten();
    let mut buffer = String::new();
    pulldown_cmark_to_cmark::cmark_with_options(events,
                                                &mut buffer,
                                                None,
                                                bilingual::md::cmark_to_cmark_opt()).unwrap();
    // std::fs::write("assets/tmp/table.md", buffer.as_bytes()).unwrap();
    assert_debug_snapshot!(buffer, @r###""|Option\tOption|Description\tDescription|\n|-----:|----------:|\n|data\tdata|path to data files to supply the data that will be passed into templates.\tpath to data files to supply the data that will be passed into templates.|\n|engine\tengine|engine to be used for processing templates. Handlebars is the default.\tengine to be used for processing templates. Handlebars is the default.|\n|ext\text|extension to be used for dest files.\textension to be used for dest files.|""###);
    assert_display_snapshot!(buffer, @r###"
    |Option	Option|Description	Description|
    |-----:|----------:|
    |data	data|path to data files to supply the data that will be passed into templates.	path to data files to supply the data that will be passed into templates.|
    |engine	engine|engine to be used for processing templates. Handlebars is the default.	engine to be used for processing templates. Handlebars is the default.|
    |ext	ext|extension to be used for dest files.	extension to be used for dest files.|
    "###);
}
