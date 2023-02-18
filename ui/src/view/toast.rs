use crate::error::Error;
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

pub struct OpenToast {
    pub text: String,
    pub title: String,
    pub info: String,
}

#[derive(Clone)]
pub enum Msg {
    ClickedOpenToast(usize),
    ClickedClose(usize),
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

    pub fn from_error(error: Error) -> Toast {
        Toast::init("error", error.title.as_str())
            .error()
            .with_more_info(error.msg)
    }

    pub fn to_open_toast(&self) -> Option<OpenToast> {
        self.more_info.clone().map(|more_info| OpenToast {
            text: self.text.clone(),
            title: self.title.clone(),
            info: more_info,
        })
    }

    fn with_variant(mut self, variant: Variant) -> Toast {
        self.variant = variant;
        self
    }

    pub fn with_more_info(mut self, more_info: impl ToString) -> Toast {
        self.more_info = Some(more_info.to_string());
        self
    }

    pub fn error(self) -> Toast {
        self.with_variant(Variant::Error)
    }

    pub fn validation_error(text: &str) -> Toast {
        Toast::init("validation error", text).error()
    }

    fn to_cell(&self, hide: bool, index: usize) -> Cell<Msg> {
        let implode_style = if hide { Style::Implode } else { Style::none() };

        let text_cell = Cell::from_str(vec![], self.text.as_str());

        let close_button = Button::simple("close")
            .on_click(move |_| Msg::ClickedClose(index))
            .cell();

        let open_button = if self.more_info.is_some() {
            Button::simple("open")
                .on_click(move |_| Msg::ClickedOpenToast(index))
                .cell()
        } else {
            Cell::none()
        };

        let more_info_row =
            Row::from_cells(vec![Style::MT4, Style::G4], vec![close_button, open_button]);

        Card::init()
            .with_header(Header::from_title(self.title.as_str()))
            .problem(self.variant == Variant::Error)
            .cell(
                vec![implode_style],
                vec![Row::from_cells(vec![], vec![text_cell]), more_info_row],
            )
    }

    pub fn many_to_html(hide_first: bool, toasts: &[Toast]) -> Node<Msg> {
        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("toasts")));

        if let Some((first, rest)) = toasts.split_first() {
            element.children.push(first.to_cell(hide_first, 0).html());

            for (index, toast) in rest.iter().enumerate() {
                element
                    .children
                    .push(toast.to_cell(false, index + 1).html());
            }
        }

        Node::Element(element)
    }
}
