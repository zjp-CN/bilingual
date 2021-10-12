use insta::assert_yaml_snapshot;
use pulldown_cmark::{CowStr, Event, Parser, Tag};

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
        _ => *include,
    }
}

pub fn extract<'a>(event: Event<'a>) -> CowStr<'a> {
    match event {
        Event::End(Tag::Heading(_)) | Event::End(Tag::Paragraph) => "\n".into(),
        // Event::Text(x) | Event::Code(x) => x,
        Event::Text(x) => x,
        Event::Code(x) => format!("\n{}\n", x).into(),
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
