use crate::style::Style;
use crate::view::card::Card;
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
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

impl Toast {
    pub fn from_str(text: &str) -> Toast {
        Toast {
            text: text.to_string(),
        }
    }

    fn to_cell<Msg: 'static>(&self, hide: bool) -> Cell<Msg> {
        let hide_style = if hide { Style::Hide } else { Style::none() };

        Card::init().cell(
            vec![hide_style],
            vec![Row::from_cells(
                vec![],
                vec![Cell::from_str(vec![], self.text.as_str())],
            )],
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
