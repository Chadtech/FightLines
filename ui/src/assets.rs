use crate::view::cell::Cell;
use crate::Style;
use seed::log;
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
    pub grass_tile: HtmlImageElement,
    pub infantry1: HtmlImageElement,
    pub infantry1_l: HtmlImageElement,
    pub infantry2: HtmlImageElement,
    pub infantry2_l: HtmlImageElement,
    pub infantry3: HtmlImageElement,
    pub infantry3_l: HtmlImageElement,
    pub infantry4: HtmlImageElement,
    pub infantry4_l: HtmlImageElement,
}

#[derive(Clone)]
pub enum Msg {}

///////////////////////////////////////////////////////////////
// INIT //
///////////////////////////////////////////////////////////////

pub fn init() -> Result<Model, String> {
    let window = web_sys::window().ok_or("Cannot find window".to_string())?;

    let doc = window.document().ok_or("Cannot get document".to_string())?;

    let get_asset = |sprite: Sprite| -> Result<HtmlImageElement, String> {
        match doc.get_element_by_id(sprite.html_id().as_str()) {
            Some(el) => Ok(HtmlImageElement::from(JsValue::from(el))),
            None => {
                log!("Here");
                let err_msg = vec![
                    "Cannot find",
                    sprite.to_human_readable_label().as_str(),
                    "asset",
                ]
                .join(" ")
                .to_string();

                Err(err_msg)
            }
        }
    };

    let grass_tile: HtmlImageElement = get_asset(Sprite::GrassTile)?;

    let infantry1: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F1,
        dir: FacingDirection::Right,
    })?;

    let infantry1_l: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F1,
        dir: FacingDirection::Left,
    })?;

    let infantry2: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F2,
        dir: FacingDirection::Right,
    })?;

    let infantry2_l: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F2,
        dir: FacingDirection::Left,
    })?;

    let infantry3: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F3,
        dir: FacingDirection::Right,
    })?;

    let infantry3_l: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F3,
        dir: FacingDirection::Left,
    })?;

    let infantry4: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F4,
        dir: FacingDirection::Right,
    })?;

    let infantry4_l: HtmlImageElement = get_asset(Sprite::Infantry {
        frame: FrameCount::F4,
        dir: FacingDirection::Left,
    })?;

    Ok(Model {
        grass_tile,
        infantry1_l,
        infantry1,
        infantry2_l,
        infantry2,
        infantry3_l,
        infantry3,
        infantry4_l,
        infantry4,
    })
}

///////////////////////////////////////////////////////////////
// VIEW //
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
