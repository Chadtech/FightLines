use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::{Card, Header};
use crate::view::cell::{Cell, Row};
use seed::dom_entity_names::Tag;
use seed::prelude::{El, Node};
use std::borrow::Cow;

////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Toast {
    text: String,
    title: String,
    more_info: Option<String>,
    variant: Variant,
}

#[derive(Clone, PartialEq)]
enum Variant {
    Normal,
    Error,
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

impl Toast {
    pub fn init(title: &str, text: &str) -> Toast {
        Toast {
            title: title.to_string(),
            text: text.to_string(),
            variant: Variant::Normal,
            more_info: None,
        }
    }

    fn with_variant(mut self, variant: Variant) -> Toast {
        self.variant = variant;
        self
    }

    pub fn with_more_info(mut self, more_info: &str) -> Toast {
        self.more_info = Some(more_info.to_string());
        self
    }

    pub fn error(mut self) -> Toast {
        self.with_variant(Variant::Error)
    }

    fn to_cell<Msg: 'static>(&self, hide: bool) -> Cell<Msg> {
        let implode_style = if hide { Style::Implode } else { Style::none() };

        let text_cell = Cell::from_str(vec![], self.text.as_str());

        let more_info_row = if self.more_info.is_some() {
            Row::from_cells(vec![Style::MT4], vec![Button::simple("open").cell()])
        } else {
            Row::none()
        };

        Card::init()
            .with_header(Header::from_title(self.title.as_str()))
            .problem(self.variant == Variant::Error)
            .cell(
                vec![implode_style],
                vec![Row::from_cells(vec![], vec![text_cell]), more_info_row],
            )
    }

    pub fn many_to_html<Msg: 'static>(hide_first: bool, toasts: &Vec<Toast>) -> Node<Msg> {
        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("toasts")));

        if let Some((first, rest)) = toasts.split_first() {
            element.children.push(first.to_cell(hide_first).html());

            for toast in rest {
                element.children.push(toast.to_cell(false).html());
            }
        }

        Node::Element(element)
    }
}
