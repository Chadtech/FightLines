use crate::style::Style::Batch;
use crate::view::text::text;
use seed::dom_entity_names::Tag;
use seed::prelude::{El, Node};
use std::borrow::Cow;

#[derive(Clone)]
#[allow(dead_code)]
pub enum Style {
    Batch(Vec<Style>),
    FlexCol,
    FlexRow,
    JustifyCenter,
    JustifyEnd,
    Grow,
    BorderR,
    P2,
    P3,
    P4,
    G3,
    G4,
    TextImportant4,
    TextContent5,
    W7,
    W8,
    W9,
    WA,
    WFull,
    BgContent0,
    BgContent1,
    BgContent4,
    Outset,
    Inset,
    BgBackground0,
    BgBackground1,
    H4,
    H5,
    H8,
    HFull,
    Relative,
    Absolute,
    OverflowHidden,
    OutlineNone,
    Pointer,
}

impl Style {
    pub fn none() -> Style {
        Batch(vec![])
    }

    pub fn css_classes(self) -> Vec<&'static str> {
        match self {
            Style::FlexCol => vec!["flex-col"],
            Style::FlexRow => vec!["flex-row"],
            Style::JustifyCenter => vec!["justify-center"],
            Style::JustifyEnd => vec!["justify-end"],
            Style::Grow => vec!["grow"],
            Style::BorderR => vec!["border-r"],
            Style::P2 => vec!["p-2"],
            Style::P3 => vec!["p-3"],
            Style::P4 => vec!["p-4"],
            Style::G3 => vec!["g-3"],
            Style::G4 => vec!["g-4"],
            Style::Batch(styles) => styles
                .into_iter()
                .map(|style| style.css_classes())
                .collect::<Vec<Vec<&'static str>>>()
                .concat(),
            Style::TextImportant4 => vec!["text-important-4"],
            Style::TextContent5 => vec!["text-content-5"],
            Style::W7 => vec!["w-7"],
            Style::W8 => vec!["w-8"],
            Style::W9 => vec!["w-9"],
            Style::WA => vec!["w-a"],
            Style::WFull => vec!["w-full"],
            Style::BgContent0 => vec!["bg-content-0"],
            Style::BgContent1 => vec!["bg-content-1"],
            Style::BgContent4 => vec!["bg-content-4"],
            Style::Outset => vec!["outset"],
            Style::Inset => vec!["inset"],
            Style::BgBackground0 => vec!["bg-background-0"],
            Style::BgBackground1 => vec!["bg-background-1"],
            Style::H4 => vec!["h-4"],
            Style::H5 => vec!["h-5"],
            Style::H8 => vec!["h-8"],
            Style::HFull => vec!["h-full"],
            Style::Relative => vec!["relative"],
            Style::Absolute => vec!["absolute"],
            Style::OverflowHidden => vec!["overflow-hidden"],
            Style::OutlineNone => vec!["outline-none"],
            Style::Pointer => vec!["pointer"],
        }
    }
}

const GLOBAL_STYLING: &str = include_str!("style.css");

pub fn global_html<Msg>() -> Node<Msg> {
    let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("style")));

    element.children.push(text(GLOBAL_STYLING));

    Node::Element(element)
}
