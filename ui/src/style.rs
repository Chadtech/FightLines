use crate::style::Style::Batch;
use crate::view::text::text;
use seed::dom_entity_names::Tag;
use seed::prelude::{El, Node};
use std::borrow::Cow;

#[derive(Clone)]
pub enum Style {
    Batch(Vec<Style>),
    FlexCol,
    FlexRow,
    JustifyCenter,
    Grow,
    BorderR,
    P2,
    P3,
    G3,
    TextImportant4,
    TextContent5,
    W8,
    WFull,
}

impl Style {
    pub fn none() -> Style {
        Batch(vec![])
    }

    pub fn to_css_classes(self) -> Vec<&'static str> {
        match self {
            Style::FlexCol => vec!["flex-col"],
            Style::FlexRow => vec!["flex-row"],
            Style::JustifyCenter => vec!["justify-center"],
            Style::Grow => vec!["grow"],
            Style::BorderR => vec!["border-r"],
            Style::P2 => vec!["p-2"],
            Style::P3 => vec!["p-3"],
            Style::G3 => vec!["g-3"],
            Style::Batch(styles) => styles
                .into_iter()
                .map(|style| style.to_css_classes())
                .collect::<Vec<Vec<&'static str>>>()
                .concat(),
            Style::TextImportant4 => vec!["text-important-4"],
            Style::TextContent5 => vec!["text-content-5"],
            Style::W8 => vec!["w-8"],
            Style::WFull => vec!["w-full"],
        }
    }
}

const GLOBAL_STYLING: &str = include_str!("style.css");

pub fn global_html<Msg>() -> Node<Msg> {
    let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("style")));

    element.children.push(text(GLOBAL_STYLING));

    Node::Element(element)
}
