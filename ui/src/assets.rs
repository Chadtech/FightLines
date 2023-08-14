use crate::page::game::view_style::ViewStyle;
use crate::view::cell::Cell;
use crate::Style;
use seed::prelude::{El, JsValue, Node, Tag};
use shared::arrow::Arrow;
use shared::facing_direction::FacingDirection;
use shared::tile;
use shared::tile::Tile;
use std::borrow::Cow;
use web_sys::HtmlImageElement;
///////////////////////////////////////////////////////////////
// Types //
///////////////////////////////////////////////////////////////

pub struct Model {
    pub sheet: Sheet,
    pub sheet_flipped: Sheet,
}

pub struct Sheet(HtmlImageElement);

pub enum MiscSpriteRow {
    GrassPlain,
    Hills,
    Forest,
    MobilitySpace,
    Arrow { arrow: ArrowRow, moved: bool },
    Cursor(ViewStyle),
    PartiallyLoadedCargoIndicator,
    FullyLoadedCargoIndicator,
    LowSuppliesIndicator,
    FogOfWar,
}

pub enum ArrowRow {
    EndLeft,
    EndDown,
    EndRight,
    EndUp,
    X,
    Y,
    RightUp,
    RightDown,
    LeftUp,
    LeftDown,
}

#[derive(Clone)]
pub enum Msg {}

const SHEET_HTML_ID: &str = "sheet";
const SHEET_FLIPPED_HTML_ID: &str = "sheet-flipped";

const MISC_SPRITE_SHEET_COLUMN: f64 = 128.0;
pub const SPRITE_SHEET_WIDTH: f64 = 10.0;

impl Sheet {
    pub fn to_html(&self) -> &HtmlImageElement {
        &self.0
    }

    pub fn draw(
        &self,
        ctx: &web_sys::CanvasRenderingContext2d,
        view_style: &ViewStyle,
        sx: f64,
        sy: f64,
        x: f64,
        y: f64,
    ) -> Result<(), JsValue> {
        let multiplier = match view_style {
            ViewStyle::Normal => 1.0,
            ViewStyle::TinySpacedUnits => 2.0,
            ViewStyle::SpacedUnits => 2.0,
        };

        let adjustment_x = match view_style {
            ViewStyle::Normal => 0.0,
            ViewStyle::TinySpacedUnits => tile::PIXEL_WIDTH_FL / 2.0,
            ViewStyle::SpacedUnits => tile::PIXEL_WIDTH_FL / 2.0,
        };

        let adjustment_y = match view_style {
            ViewStyle::Normal => 0.0,
            ViewStyle::TinySpacedUnits => tile::PIXEL_HEIGHT_FL / 2.0,
            ViewStyle::SpacedUnits => tile::PIXEL_HEIGHT_FL / 2.0,
        };

        ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            self.to_html(),
            sx,
            sy,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
            x * multiplier + adjustment_x,
            y * multiplier + adjustment_y,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
        )
    }
}

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
            sheet: Sheet(sheet),
            sheet_flipped: Sheet(sheet_flipped),
        })
    }

    pub fn sheet_from_facing_dir(&self, facing_dir: &FacingDirection) -> &Sheet {
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
        self.draw_misc_sprite_precise(ctx, misc_sprite_row, x as f64, y as f64, 0.0, 0.0)
    }

    pub fn draw_misc_sprite_precise(
        &self,
        ctx: &web_sys::CanvasRenderingContext2d,
        misc_sprite_row: MiscSpriteRow,
        x: f64,
        y: f64,
        x_adjusmentt: f64,
        y_adjustment: f64,
    ) -> Result<(), JsValue> {
        let x = x * tile::PIXEL_WIDTH_FL;
        let y = y * tile::PIXEL_HEIGHT_FL;

        ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &self.sheet.0,
            MISC_SPRITE_SHEET_COLUMN,
            misc_sprite_row.row_number() * tile::PIXEL_HEIGHT_FL,
            tile::PIXEL_WIDTH_FL,
            tile::PIXEL_HEIGHT_FL,
            x + x_adjusmentt,
            y + y_adjustment,
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
            MiscSpriteRow::Arrow { arrow, moved } => {
                let mut r = match arrow {
                    ArrowRow::EndLeft => 6.0,
                    ArrowRow::EndDown => 9.0,
                    ArrowRow::EndRight => 4.0,
                    ArrowRow::EndUp => 7.0,
                    ArrowRow::X => 5.0,
                    ArrowRow::Y => 8.0,
                    ArrowRow::RightUp => 10.0,
                    ArrowRow::RightDown => 11.0,
                    ArrowRow::LeftUp => 12.0,
                    ArrowRow::LeftDown => 13.0,
                };

                if *moved {
                    r += 10.0
                }

                r
            }
            MiscSpriteRow::Cursor(_) => 2.0,
            MiscSpriteRow::PartiallyLoadedCargoIndicator => 26.0,
            MiscSpriteRow::FullyLoadedCargoIndicator => 28.0,
            MiscSpriteRow::LowSuppliesIndicator => 27.0,
            MiscSpriteRow::FogOfWar => 1.0,
        }
    }
}

pub struct ArrowParams<'a> {
    pub arrow: &'a Arrow,
    pub moved: bool,
}

impl<'a> From<ArrowParams<'a>> for MiscSpriteRow {
    fn from(value: ArrowParams<'a>) -> Self {
        let ArrowParams { arrow, moved } = value;

        let r = match arrow {
            Arrow::EndLeft => ArrowRow::EndLeft,
            Arrow::EndDown => ArrowRow::EndDown,
            Arrow::EndRight => ArrowRow::EndRight,
            Arrow::EndUp => ArrowRow::EndUp,
            Arrow::X => ArrowRow::X,
            Arrow::Y => ArrowRow::Y,
            Arrow::RightUp => ArrowRow::RightUp,
            Arrow::RightDown => ArrowRow::RightDown,
            Arrow::LeftUp => ArrowRow::LeftUp,
            Arrow::LeftDown => ArrowRow::LeftDown,
        };

        MiscSpriteRow::Arrow { arrow: r, moved }
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
