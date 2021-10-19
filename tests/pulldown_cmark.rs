use insta::{assert_display_snapshot, assert_yaml_snapshot};
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};

pub fn filter(event: &Event, include: &mut bool) -> bool {
    match event {
        Event::Start(Tag::CodeBlock(_)) => {
            *include = false;
            false
        }
        Event::End(Tag::CodeBlock(_)) => {
            *include = true;
            false
        }
        Event::Start(Tag::Heading(_)) | Event::Start(Tag::Paragraph) => false,
        // 排除行间代码
        _ => *include,
    }
}

pub fn extract<'a>(event: Event<'a>) -> CowStr<'a> {
    match event {
        Event::Text(x) => x,
        Event::End(Tag::Heading(_)) | Event::End(Tag::Paragraph) => "\n".into(),
        Event::Code(x) => format!("\n{}\n", x).into(), // 行内代码

        _ => " ".into(),
    }
}

#[test]
fn default_config() -> std::io::Result<()> {
    let md = std::fs::read_to_string("assets/8_6_io_eventqueue.md")?;
    assert_yaml_snapshot!("8_6_io_eventqueue", Parser::new(&md).collect::<Vec<_>>());
    assert_yaml_snapshot!("8_6_io_eventqueue_offset",
                          Parser::new(&md).into_offset_iter().collect::<Vec<_>>());

    let parsed = {
        let mut include = true;
        Parser::new(&md).filter(|event| filter(event, &mut include))
                        .map(extract)
                        .collect::<Vec<_>>()
    };
    // assert_yaml_snapshot!("8_6_io_eventqueue_modified", Parser::new(&md).collect::<Vec<_>>());
    assert_yaml_snapshot!("8_6_io_eventqueue_modified", parsed.join(""));

    assert_yaml_snapshot!("8_6_io_eventqueue_translated",
        Parser::new(
            &std::fs::read_to_string("assets/8_6_io_eventqueue_translated.md")?
        ).collect::<Vec<_>>()
    );

    Ok(())
}

fn full_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    // options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_TABLES);
    options
}

#[test]
fn full_config() -> std::io::Result<()> {
    let file = "markdown-it";
    let md = std::fs::read_to_string(format!("assets/{}.md", file))?;
    let opt = full_options();
    assert_yaml_snapshot!(file, Parser::new_ext(&md, opt).collect::<Vec<_>>());
    assert_yaml_snapshot!(format!("{}_offset", file),
                          Parser::new_ext(&md, opt).into_offset_iter().collect::<Vec<_>>());

    let parsed = {
        let mut include = true;
        Parser::new_ext(&md, opt).filter(|event| filter(event, &mut include))
                                 .map(extract)
                                 .collect::<Vec<_>>()
    };
    // assert_yaml_snapshot!("8_6_io_eventqueue_modified", Parser::new(&md).collect::<Vec<_>>());
    assert_display_snapshot!(format!("{}_modified", file), parsed.join(""));

    Ok(())
}
