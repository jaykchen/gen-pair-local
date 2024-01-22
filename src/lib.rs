use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionResponseFormat, ChatCompletionResponseFormatType,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use pandoc_types::definition::Block;
use pandoc_types::definition::Inline;
use pandoc_types::definition::*;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;

fn stringify_inline(inline: &Inline) -> String {
    match inline {
        Inline::Str(text) => text.clone(),
        Inline::Emph(children)
        | Inline::Strong(children)
        | Inline::Strikeout(children)
        | Inline::Superscript(children)
        | Inline::Subscript(children)
        | Inline::SmallCaps(children)
        | Inline::Quoted(_, children)
        | Inline::Cite(_, children)
        | Inline::Link(_, children, _)
        | Inline::Image(_, children, _) => children.iter().map(stringify_inline).collect(),
        Inline::Space => " ".to_string(),
        Inline::SoftBreak => "\n".to_string(),
        Inline::LineBreak => "\n".to_string(),
        _ => String::new(),
    }
}

fn stringify_block(block: &Block) -> String {
    match block {
        Block::Plain(children) | Block::Para(children) => {
            children.iter().map(stringify_inline).collect()
        }
        Block::LineBlock(lines) => lines
            .iter()
            .map(|line| line.iter().map(stringify_inline).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n"),
        Block::CodeBlock(_, code) => code.clone(),
        Block::RawBlock(_, raw) => raw.clone(),
        Block::BlockQuote(children) | Block::Div(_, children) => {
            children.iter().map(stringify_block).collect()
        }
        Block::BulletList(items) | Block::OrderedList(_, items) => items
            .iter()
            .map(|item| item.iter().map(stringify_block).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n"),
        Block::DefinitionList(items) => items
            .iter()
            .map(|(term, definitions)| {
                format!(
                    "{}\n{}",
                    term.iter().map(stringify_inline).collect::<String>(),
                    definitions
                        .iter()
                        .map(|definition| {
                            definition.iter().map(stringify_block).collect::<String>()
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Block::Header(_, _, children) => children.iter().map(stringify_inline).collect(),
        _ => String::new(),
    }
}

pub struct Document {
    collected_texts: Vec<Vec<String>>,
    current_segment: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Document {
            collected_texts: vec![],
            current_segment: vec![],
        }
    }

    pub fn action(&mut self, elem: &Block) {
        if let Block::Header(_, _, ref children) = elem {
            if !self.current_segment.is_empty() {
                self.collected_texts.push(self.current_segment.clone());
                self.current_segment.clear();
            }
        }

        match elem {
            Block::Header(_, _, ref children)
            | Block::Para(ref children)
            | Block::Plain(ref children) => {
                self.current_segment
                    .extend(children.iter().map(|child| stringify_inline(child)));
            }
            Block::LineBlock(ref lines) => {
                self.current_segment.push(
                    lines
                        .iter()
                        .map(|line| line.iter().map(stringify_inline).collect::<String>())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
            Block::BulletList(ref items) | Block::OrderedList(_, ref items) => {
                self.current_segment.extend(
                    items
                        .iter()
                        .map(|item| item.iter().map(stringify_block).collect::<String>()),
                );
            }
            Block::BlockQuote(ref children) | Block::Div(_, ref children) => {
                self.current_segment
                    .extend(children.iter().map(stringify_block));
            }
            Block::CodeBlock(_, code) => {
                self.current_segment.push(code.clone());
            }
            Block::RawBlock(_, content) => {
                self.current_segment.push(content.clone());
            }
            _ => {}
        }
    }

    pub fn finalize(&mut self) {
        if !self.current_segment.is_empty() {
            self.collected_texts.push(self.current_segment.clone());
        }

        let json_data = json!(self.collected_texts);
        let mut file = File::create("segmented_text.json").expect("Failed to create file");
        file.write_all(json_data.to_string().as_bytes())
            .expect("Failed to write to file");
    }
}

pub fn stringify_inline(inline: &Inline) -> String {
    match inline {
        Inline::Str(text) => text.clone(),
        Inline::Space => " ".to_string(),
        Inline::Emph(inlines) => {
            format!(
                "*{}*",
                inlines
                    .iter()
                    .map(stringify_inline)
                    .collect::<Vec<_>>()
                    .join("")
            )
        }
        Inline::Strong(inlines) => {
            format!(
                "**{}**",
                inlines
                    .iter()
                    .map(stringify_inline)
                    .collect::<Vec<_>>()
                    .join("")
            )
        }
        _ => String::new(),
    }
}

pub fn stringify_block(block: &Block) -> String {
    match block {
        Block::Plain(children) | Block::Para(children) => children
            .iter()
            .map(stringify_inline)
            .collect::<Vec<_>>()
            .join(" "),
        Block::LineBlock(lines) => lines
            .iter()
            .map(|line| line.iter().map(stringify_inline).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n"),
        Block::CodeBlock(CodeBlock(_, code)) => {
            format!("```\n{}\n```\n", code)
        }
        Block::RawBlock(_, raw) => raw.clone(),
        Block::BlockQuote(children) | Block::Div(_, children) => {
            let quoted_text = children
                .iter()
                .map(stringify_block)
                .collect::<Vec<_>>()
                .join("\n\n");
            format!("> {}\n", quoted_text)
        }
        Block::BulletList(items) | Block::OrderedList(OrderedList(_, items)) => {
            let list_items = items
                .iter()
                .map(|item| {
                    let item_text = item.iter().map(stringify_block).collect::<String>();
                    format!("* {}", item_text)
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n{}\n", list_items)
        }
        Block::Header(_, _, children) => {
            let header_text = children
                .iter()
                .map(stringify_inline)
                .collect::<Vec<_>>()
                .join(" ");
            format!("{}\n\n", header_text)
        }
        Block::DefinitionList(items) => items
            .iter()
            .map(|(term, definitions)| {
                format!(
                    "{}\n:\t{}",
                    term.iter().map(stringify_inline).collect::<String>(),
                    definitions
                        .iter()
                        .map(|definition| {
                            definition.iter().map(stringify_block).collect::<String>()
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
        // You can add more match arms for other `Block` variants if needed
        _ => String::new(),
    }
}
