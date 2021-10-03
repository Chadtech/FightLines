use crate::route;
use crate::view::button::Click::NoClick;
use crate::view::cell::Cell;
use crate::view::text::text;
use seed::dom_entity_names::Tag;
use seed::prelude::*;
use seed::prelude::{El, Node};
use std::borrow::Cow;
use std::rc::Rc;
use web_sys::MouseEvent;

////////////////////////////////////////////////////////////////
// Types //
////////////////////////////////////////////////////////////////

pub struct Button<Msg: 'static> {
    label: String,
    on_click: Click<Msg>,
    active: bool,
    variant: Variant,
}

enum Variant {
    Simple,
    Primary,
}

enum Click<Msg> {
    NoClick,
    Handler(Rc<dyn Fn(MouseEvent) -> Msg>),
    Route(route::Route),
}

////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////

impl Variant {
    fn to_css_class(self) -> &'static str {
        match self {
            Variant::Simple => "button-simple",
            Variant::Primary => "button-primary",
        }
    }
}

////////////////////////////////////////////////////////////////
// Api //
////////////////////////////////////////////////////////////////

impl<Msg: 'static> Button<Msg> {
    fn from_variant(label: &str, variant: Variant) -> Button<Msg> {
        Button {
            label: label.to_string(),
            on_click: NoClick,
            active: false,
            variant,
        }
    }
    pub fn primary(label: &str) -> Button<Msg> {
        Button::from_variant(label, Variant::Primary)
    }
    pub fn simple(label: &str) -> Button<Msg> {
        Button::from_variant(label, Variant::Simple)
    }
    pub fn active(mut self, active: bool) -> Button<Msg> {
        self.active = active;
        self
    }
    pub fn route(mut self, route: route::Route) -> Button<Msg> {
        self.on_click = Click::Route(route);
        self
    }
    pub fn to_cell(self) -> Cell<Msg> {
        Cell::from_html(vec![], vec![self.to_html()])
    }
    pub fn to_html(self) -> Node<Msg> {
        let tag = match self.on_click {
            Click::Route(_) => "a",
            _ => "button",
        };

        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed(tag)));

        element.add_class("button");

        element.add_class(self.variant.to_css_class());

        if self.active {
            element.add_class("active");
        }

        element.children.push(text(self.label.as_str()));

        match self.on_click {
            Click::Handler(on_click) => {
                element.add_event_handler(mouse_ev(Ev::Click, move |event| on_click(event)));
            }
            Click::Route(route) => {
                element.add_attr(Cow::Borrowed("href"), route.to_string());
            }
            NoClick => {}
        }

        Node::Element(element)
    }
}
