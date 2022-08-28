use crate::view::cell::Cell;
use seed::prelude::{El, Node, Tag};
use shared::sprite::Sprite;
use std::borrow::Cow;
///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Assets {}

impl Assets {
    pub fn init() -> Assets {
        Assets {}
    }
}

#[derive(Clone)]
pub enum Msg {}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

pub fn view() -> Cell<Msg> {
    Cell::from_html(vec![], vec![image_view(Sprite::GrassTile)])
}

fn image_view(sprite: Sprite) -> Node<Msg> {
    let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("img")));

    element.add_attr("src", sprite.to_file_name().as_str());
    element.add_attr("id", sprite.html_id().as_str());

    Node::Element(element)
}
