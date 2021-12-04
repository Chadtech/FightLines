use crate::view::cell::Cell;
use seed::dom_entity_names::Tag;
use seed::prelude::*;
use std::borrow::Cow;
use std::rc::Rc;

////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TextField<Msg: 'static> {
    value: String,
    placeholder: Option<String>,
    on_input: Rc<dyn Fn(String) -> Msg>,
}

////////////////////////////////////////////////////////////////
// API //
////////////////////////////////////////////////////////////////

impl<Msg: 'static> TextField<Msg> {
    pub fn simple(
        value: &str,
        on_input: impl FnOnce(String) -> Msg + Clone + 'static,
    ) -> TextField<Msg> {
        TextField {
            value: value.to_string(),
            placeholder: None,
            on_input: Rc::new(move |event| on_input.clone()(event)),
        }
    }
    pub fn html(self) -> Node<Msg> {
        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("input")));

        let on_input = self.on_input;

        element.add_event_handler(input_ev(Ev::Input, move |event| on_input(event)));

        element.add_class("text-field");

        element.add_attr(Cow::Borrowed("value"), self.value);

        match self.placeholder {
            None => {}
            Some(placeholder_text) => {
                element.add_attr(Cow::Borrowed("placeholder"), placeholder_text);
            }
        }

        Node::Element(element)
    }

    pub fn cell(self) -> Cell<Msg> {
        Cell::from_html(vec![], vec![self.html()])
    }

    pub fn placeholder(mut self, placeholder_text: &str) -> TextField<Msg> {
        self.placeholder = Some(placeholder_text.to_string());
        self
    }
}
