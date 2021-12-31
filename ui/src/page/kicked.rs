use crate::core_ext::route::go_to_route;
use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::cell::Row;
use crate::view::error_card::ErrorCard;
use seed::prelude::Orders;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum Msg {
    ClickedGoBackToTitle,
}

///////////////////////////////////////////////////////////////
// Update //
///////////////////////////////////////////////////////////////

pub fn update(msg: Msg, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClickedGoBackToTitle => {
            go_to_route(orders, Route::Title);
        }
    }
}

///////////////////////////////////////////////////////////////
// View //
///////////////////////////////////////////////////////////////

pub fn view() -> Vec<Row<Msg>> {
    let card = ErrorCard::from_title("kicked")
        .with_buttons(vec![
            Button::primary("go back to title page").on_click(|_| Msg::ClickedGoBackToTitle)
        ])
        .with_msg("you were kicked from the lobby by the host");

    vec![Row::from_cells(
        vec![Style::JustifyCenter],
        vec![card.cell()],
    )]
}

pub const PARENT_STYLES: [Style; 1] = [Style::JustifyCenter];
