use crate::route::Route;
use crate::style::Style;
use crate::view::button::Button;
use crate::view::card;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::view::loading_spinner::LoadingSpinner;
use crate::view::textarea::Textarea;
use crate::{core_ext, global};
use seed::log;
use seed::prelude::{fetch, Method, Orders, Request};
use shared::api::create_lobby;
use shared::api::create_lobby::Response;
use shared::api::endpoint::Endpoint;
use shared::id::Id;

///////////////////////////////////////////////////////////////
// Types
///////////////////////////////////////////////////////////////

pub struct Model;

#[derive(Clone, Debug)]
pub enum Msg {
    Msg,
}

///////////////////////////////////////////////////////////////
// Init
///////////////////////////////////////////////////////////////

pub fn init() -> Model {
    Model
}

///////////////////////////////////////////////////////////////
// Update
///////////////////////////////////////////////////////////////

pub fn update(global: &global::Model, msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Msg => {}
    }
}

///////////////////////////////////////////////////////////////
// View
///////////////////////////////////////////////////////////////

pub fn view(model: &Model) -> Vec<Row<Msg>> {
    vec![]
}
