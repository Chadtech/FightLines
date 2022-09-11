use crate::view::cell::Cell;
use crate::Style;
use seed::prelude::{El, Node, Tag};
use shared::sprite::Sprite;
use shared::{facing_direction, frame_count};
use std::borrow::Cow;
///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum Msg {}

///////////////////////////////////////////////////////////////
// Api //
///////////////////////////////////////////////////////////////

pub fn view() -> Cell<Msg> {
    let mut infantry_images = vec![];

    for dir in facing_direction::ALL {
        for frame in frame_count::ALL {
            infantry_images.push(image_view(Sprite::Infantry {
                frame: frame.clone(),
                dir: dir.clone(),
            }))
        }
    }

    Cell::from_html(
        vec![Style::Hide],
        vec![vec![image_view(Sprite::GrassTile)], infantry_images].concat(),
    )
}

fn image_view(sprite: Sprite) -> Node<Msg> {
    let mut element: El<Msg> = El::empty(Tag::Custom(Cow::Borrowed("img")));

    let mut file_name = "/".to_string();
    file_name.push_str(sprite.to_file_name().as_str());

    element.add_attr("src", file_name);
    element.add_attr("id", sprite.html_id().as_str());

    Node::Element(element)
}
