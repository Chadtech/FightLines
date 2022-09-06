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
    BorderContent1,
    BorderContent2,
    P2,
    P3,
    P4,
    G3,
    G4,
    M2,
    MT4,
    TextImportant4,
    TextContent1,
    TextContent5,
    W7,
    W8,
    W9,
    WA,
    W512px,
    WFull,
    BgContent0,
    BgContent1,
    BgContent4,
    Outset,
    Inset,
    InsetImportant,
    BgBackground0,
    BgBackground1,
    BgImportant1,
    BgProblem1,
    H4,
    H5,
    H8,
    H512px,
    HFull,
    Relative,
    Absolute,
    OverflowHidden,
    OutlineNone,
    Pointer,
    Hide,
    Implode,
    Screen,
    AbsoluteCenter,
    Left0,
    Right0,
    Top0,
    Bottom0,
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
            Style::BorderContent1 => vec!["border-content-1"],
            Style::BorderContent2 => vec!["border-content-2"],
            Style::P2 => vec!["p-2"],
            Style::P3 => vec!["p-3"],
            Style::P4 => vec!["p-4"],
            Style::G3 => vec!["g-3"],
            Style::G4 => vec!["g-4"],
            Style::M2 => vec!["m-2"],
            Style::MT4 => vec!["mt-4"],
            Style::Batch(styles) => styles
                .into_iter()
                .map(|style| style.css_classes())
                .collect::<Vec<Vec<&'static str>>>()
                .concat(),
            Style::TextImportant4 => vec!["text-important-4"],
            Style::TextContent1 => vec!["text-content-1"],
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
            Style::InsetImportant => vec!["inset-important"],
            Style::BgBackground0 => vec!["bg-background-0"],
            Style::BgBackground1 => vec!["bg-background-1"],
            Style::BgProblem1 => vec!["bg-problem-1"],
            Style::H4 => vec!["h-4"],
            Style::H5 => vec!["h-5"],
            Style::H8 => vec!["h-8"],
            Style::HFull => vec!["h-full"],
            Style::Relative => vec!["relative"],
            Style::Absolute => vec!["absolute"],
            Style::OverflowHidden => vec!["overflow-hidden"],
            Style::OutlineNone => vec!["outline-none"],
            Style::Pointer => vec!["pointer"],
            Style::BgImportant1 => vec!["bg-important-1"],
            Style::Hide => vec!["hide"],
            Style::Implode => vec!["implode"],
            Style::Screen => vec!["screen"],
            Style::AbsoluteCenter => vec!["absolute-center"],
            Style::Left0 => vec!["left-0"],
            Style::Right0 => vec!["right-0"],
            Style::Top0 => vec!["top-0"],
            Style::Bottom0 => vec!["bottom-0"],
            Style::W512px => vec!["w-512px"],
            Style::H512px => vec!["h-512px"],
        }
    }
}

const GLOBAL_STYLING: &str = include_str!("style.css");

pub fn global_html<Msg>() -> Node<Msg> {
    let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("style")));

    element.children.push(text(GLOBAL_STYLING));

    Node::Element(element)
}
