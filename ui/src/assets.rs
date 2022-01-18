use crate::view::cell::Cell;
use seed::img;
use seed::prelude::{ev, El, Node, Tag};
use shared::api::endpoint::Endpoint;
use std::borrow::Cow;
///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Assets {}

#[derive(Clone)]
pub enum Msg {}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

impl Assets {
    pub fn init() -> Assets {
        Assets {}
    }
}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

pub fn view() -> Cell<Msg> {
    Cell::from_html(
        vec![],
        vec![image_view(Endpoint::GrassTileAsset.to_string().as_str())],
    )
}

fn image_view(src: &str) -> Node<Msg> {
    let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("img")));

    element.add_attr("src", src);

    Node::Element(element)
}
