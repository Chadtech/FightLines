use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::Row;
use crate::view::error_card::ErrorCard;
use seed::prelude::Orders;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub struct Model {
    title: String,
    msg: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Msg {
    ClickedGoBackToTitle,
}

pub struct Flags {
    title: String,
    msg: Option<String>,
}

impl Flags {
    pub fn from_title(title: String) -> Flags {
        Flags { title, msg: None }
    }

    pub fn with_msg(mut self, msg: String) -> Flags {
        self.msg = Some(msg);

        self
    }
}

///////////////////////////////////////////////////////////////
// Init //
///////////////////////////////////////////////////////////////

pub fn init(flags: Flags) -> Model {
    Model {
        title: flags.title,
        msg: flags.msg,
    }
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(msg: Msg, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClickedGoBackToTitle => {
            orders.request_url(Route::Title.to_url());
        }
    }
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view(model: &Model) -> Vec<Row<Msg>> {
    let mut card = ErrorCard::from_title(model.title.as_str())
        .with_buttons(vec![
            Button::primary("go back to title page").on_click(|_| Msg::ClickedGoBackToTitle)
        ]);

    if let Some(msg) = &model.msg {
        card = card.with_msg(msg);
    }

    vec![Row::from_cells(
        vec![Style::JustifyCenter],
        vec![card.cell()],
    )]
}

pub const PARENT_STYLES: [Style; 1] = [Style::JustifyCenter];
