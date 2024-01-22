use pandoc_ast::{ Inline, Block };

pub fn stringify_inline(inline: &Inline) -> String {
    match inline {
        Inline::Str(text) => text.clone(),
        | Inline::Emph(children)
        | Inline::Strong(children)
        | Inline::Strikeout(children)
        | Inline::Superscript(children)
        | Inline::Subscript(children)
        | Inline::SmallCaps(children)
        | Inline::Quoted(_, children)
        | Inline::Cite(_, children)
        | Inline::Link(_, children, _)
        | Inline::Image(_, children, _)
        | Inline::Span(_, children) => {
            children.iter().map(stringify_inline).collect::<Vec<_>>().join("")
        }
        Inline::Space => " ".to_string(),
        Inline::SoftBreak | Inline::LineBreak => "\n".to_string(),
        Inline::Code(_, code) => code.clone(),
        Inline::Math(_, content) => content.clone(),
        Inline::RawInline(_, content) => content.clone(),
        Inline::Note(blocks) => blocks.iter().map(stringify_block).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

pub fn stringify_block(block: &pandoc_ast::Block) -> String {
    match block {
        Block::Plain(children) | Block::Para(children) => {
            children.iter().map(stringify_inline).collect::<Vec<_>>().join("")
        }
        Block::LineBlock(lines) => {
            lines
                .iter()
                .map(|line| line.iter().map(stringify_inline).collect::<Vec<_>>().join(""))
                .collect::<Vec<_>>()
                .join("\n")
        }
        Block::CodeBlock(_, code) => code.clone(),
        Block::RawBlock(_, raw) => raw.clone(),
        Block::BlockQuote(children) | Block::Div(_, children) => {
            children.iter().map(stringify_block).collect::<Vec<_>>().join("\n")
        }
        Block::BulletList(items) | Block::OrderedList(_, items) => {
            items
                .iter()
                .map(|item| item.iter().map(stringify_block).collect::<Vec<_>>().join("\n"))
                .collect::<Vec<_>>()
                .join("\n")
        }
        Block::DefinitionList(items) => {
            items
                .iter()
                .map(|(term, definitions)| {
                    format!(
                        "{}\n{}",
                        term.iter().map(stringify_inline).collect::<Vec<_>>().join(""),
                        definitions
                            .iter()
                            .map(|definition|
                                definition
                                    .iter()
                                    .map(stringify_block)
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        Block::Header(_, _, children) => {
            children.iter().map(stringify_inline).collect::<Vec<_>>().join("")
        }
        Block::HorizontalRule => "-----\n".to_string(),
        Block::Table(..) | Block::Figure(..) => {
            // Handling for tables and figures would require additional logic
            // to properly format them as text.
            String::new()
        }
        Block::Null => String::new(),
    }
}
