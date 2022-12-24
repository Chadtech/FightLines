use crate::route;
use crate::style::Style;
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
    on_click: Option<Click<Msg>>,
    active: bool,
    variant: Variant,
    full_width: bool,
}

enum Variant {
    Simple,
    Primary,
    Disabled,
    Destructive,
}

enum Click<Msg> {
    Handler(Rc<dyn Fn(MouseEvent) -> Msg>),
    Route(route::Route),
}

////////////////////////////////////////////////////////////////
// Helpers //
////////////////////////////////////////////////////////////////

impl Variant {
    fn to_css_class(&self) -> &'static str {
        match self {
            Variant::Simple => "button-simple",
            Variant::Primary => "button-primary",
            Variant::Disabled => "button-disabled",
            Variant::Destructive => "button-destructive",
        }
    }

    fn to_styles(&self) -> Vec<Style> {
        match self {
            Variant::Simple => vec![Style::Pointer, Style::Outset],
            Variant::Primary => vec![Style::Pointer],
            Variant::Disabled => vec![Style::BgContent0, Style::Outset],

            Variant::Destructive => vec![Style::Pointer],
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
            on_click: None,
            active: false,
            variant,
            full_width: false,
        }
    }
    pub fn full_width(mut self) -> Button<Msg> {
        self.full_width = true;
        self
    }
    pub fn primary(label: &str) -> Button<Msg> {
        Button::from_variant(label, Variant::Primary)
    }
    pub fn simple(label: &str) -> Button<Msg> {
        Button::from_variant(label, Variant::Simple)
    }
    pub fn disabled(label: &str) -> Button<Msg> {
        Button::from_variant(label, Variant::Disabled)
    }
    pub fn destructive(label: &str) -> Button<Msg> {
        Button::from_variant(label, Variant::Destructive)
    }
    pub fn disable(mut self, d: bool) -> Button<Msg> {
        if d {
            self.variant = Variant::Disabled
        }

        self
    }
    pub fn active(mut self, active: bool) -> Button<Msg> {
        self.active = active;
        self
    }
    pub fn on_click(
        mut self,
        msg: impl FnOnce(MouseEvent) -> Msg + Clone + 'static,
    ) -> Button<Msg> {
        self.on_click = Some(Click::Handler(Rc::new(move |event| msg.clone()(event))));
        self
    }
    pub fn route(mut self, route: route::Route) -> Button<Msg> {
        self.on_click = Some(Click::Route(route));
        self
    }
    pub fn cell(self) -> Cell<Msg> {
        Cell::from_html(vec![], vec![self.html()])
    }
    pub fn html(self) -> Node<Msg> {
        let tag = match self.on_click {
            Some(Click::Route(_)) => "a",
            _ => "button",
        };

        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed(tag)));

        element.add_class("button");

        element.add_class(self.variant.to_css_class());

        for style in self.variant.to_styles() {
            for class in style.css_classes() {
                element.add_class(class);
            }
        }

        if self.active {
            element.add_class("active");
        }

        if self.full_width {
            for class in Style::WFull.css_classes() {
                element.add_class(class);
            }
        }

        element.children.push(text(self.label.as_str()));

        if let Some(on_click) = self.on_click {
            match on_click {
                Click::Handler(on_click_fn) => {
                    element.add_event_handler(mouse_ev(Ev::Click, move |event| on_click_fn(event)));
                }
                Click::Route(route) => {
                    element.add_attr(Cow::Borrowed("href"), route.to_string());
                }
            }
        }

        Node::Element(element)
    }
}
