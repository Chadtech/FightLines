use crate::view::cell::Cell;
use crate::Style;
use seed::prelude::{El, JsValue, Node, Tag};
use shared::facing_direction::FacingDirection;
use shared::tile;
use shared::tile::Tile;
use std::borrow::Cow;
use web_sys::HtmlImageElement;
///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    pub sheet: HtmlImageElement,
    pub sheet_flipped: HtmlImageElement,
}

pub enum MiscSpriteRow {
    GrassPlain,
    Hills,
    Forest,
    MobilitySpace,
    ArrowEndLeft,
    ArrowEndDown,
    ArrowEndRight,
    ArrowEndUp,
    ArrowX,
    ArrowY,
    ArrowRightUp,
    ArrowRightDown,
    ArrowLeftUp,
    ArrowLeftDown,
}

#[derive(Clone)]
pub enum Msg {}

const SHEET_HTML_ID: &str = "sheet";
const SHEET_FLIPPED_HTML_ID: &str = "sheet-flipped";

pub const MISC_SPRITE_SHEET_COLUMN: f64 = 128.0;
pub const SPRITE_SHEET_WIDTH: f64 = 10.0;

impl Model {
    pub fn init() -> Result<Model, String> {
        let window = web_sys::window().ok_or_else(|| "Cannot find window".to_string())?;

        let doc = window
            .document()
            .ok_or_else(|| "Cannot get document".to_string())?;

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

    pub fn sheet_from_facing_dir(&self, facing_dir: &FacingDirection) -> &HtmlImageElement {
        match facing_dir {
            FacingDirection::Left => &self.sheet_flipped,
            FacingDirection::Right => &self.sheet,
        }
    }

    pub fn draw_misc_sprite(
        &self,
        ctx: &web_sys::CanvasRenderingContext2d,
        misc_sprite_row: MiscSpriteRow,
        x: u16,
        y: u16,
    ) -> Result<(), JsValue> {
        ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &self.sheet,
            MISC_SPRITE_SHEET_COLUMN,
            misc_sprite_row.row_number() * tile::PIXEL_HEIGHT_FL,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
            x as f64 * tile::PIXEL_WIDTH_FL,
            y as f64 * tile::PIXEL_HEIGHT_FL,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
        )
    }
}

impl MiscSpriteRow {
    pub fn row_number(&self) -> f64 {
        match self {
            MiscSpriteRow::GrassPlain => 0.0,
            MiscSpriteRow::Hills => 24.0,
            MiscSpriteRow::Forest => 25.0,
            MiscSpriteRow::MobilitySpace => 3.0,
        }
    }
}

impl From<&Tile> for MiscSpriteRow {
    fn from(tile: &Tile) -> Self {
        match tile {
            Tile::GrassPlain => MiscSpriteRow::GrassPlain,
            Tile::Hills => MiscSpriteRow::Hills,
            Tile::Forest => MiscSpriteRow::Forest,
        }
    }
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
