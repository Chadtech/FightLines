use crate::style::Style;
use crate::view::cell::Cell;
use seed::dom_entity_names::Tag;
use seed::prelude::{El, Node};
use std::borrow::Cow;

pub struct Textarea {
    value: String,
}

impl Textarea {
    pub fn simple(value: String) -> Textarea {
        Textarea { value }
    }
    pub fn html<Msg: 'static>(self) -> Node<Msg> {
        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("textarea")));

        element.add_attr("value", self.value);

        for style in STYLES {
            for class in style.css_classes() {
                element.add_class(class);
            }
        }

        Node::Element(element)
    }

    pub fn cell<Msg: 'static>(self, styles: Vec<Style>) -> Cell<Msg> {
        Cell::from_html(styles, vec![self.html()])
    }
}

const STYLES: [Style; 6] = [
    Style::Inset,
    Style::BgBackground0,
    Style::WFull,
    Style::HFull,
    Style::P3,
    Style::OutlineNone,
];
