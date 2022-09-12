use crate::view::cell::Cell;
use crate::Style;
use seed::prelude::{El, JsValue, Node, Tag};
use shared::facing_direction::FacingDirection;
use shared::frame_count::FrameCount;
use shared::sprite::Sprite;
use shared::{facing_direction, frame_count};
use std::borrow::Cow;
use web_sys::HtmlImageElement;
///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    pub sheet: HtmlImageElement,
    pub sheet_flipped: HtmlImageElement,
}

#[derive(Clone)]
pub enum Msg {}

///////////////////////////////////////////////////////////////
// INIT //
///////////////////////////////////////////////////////////////

const SHEET_HTML_ID: &'static str = "sheet";
const SHEET_FLIPPED_HTML_ID: &'static str = "sheet-flipped";

pub fn init() -> Result<Model, String> {
    let window = web_sys::window().ok_or("Cannot find window".to_string())?;

    let doc = window.document().ok_or("Cannot get document".to_string())?;

    let sheet = {
        match doc.get_element_by_id(SHEET_HTML_ID) {
            Some(el) => Ok(HtmlImageElement::from(JsValue::from(el))),
            None => Err("Cannot find sprite sheet img"),
        }
    }?;

    let sheet_flipped = {
        match doc.get_element_by_id(SHEET_FLIPPED_HTML_ID) {
            Some(el) => Ok(HtmlImageElement::from(JsValue::from(el))),
            None => Err("Cannot find sprite sheet img"),
        }
    }?;

    Ok(Model {
        sheet,
        sheet_flipped,
    })
}

///////////////////////////////////////////////////////////////
// VIEW //
///////////////////////////////////////////////////////////////

pub fn view() -> Cell<Msg> {
    let sheet_el = {
        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("img")));

        element.add_attr("src", "/asset/sheet.png");
        element.add_attr("id", SHEET_HTML_ID);

        Node::Element(element)
    };

    let sheet_flipped_el = {
        let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("img")));

        element.add_attr("src", "/asset/sheet-flipped.png");
        element.add_attr("id", SHEET_FLIPPED_HTML_ID);

        Node::Element(element)
    };

    Cell::from_html(vec![Style::Hide], vec![sheet_el, sheet_flipped_el])
}
