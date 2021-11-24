use crate::style::Style;
use crate::view::button::Button;
use crate::view::card::Card;
use crate::view::cell::{Cell, Row};
use crate::view::textarea::Textarea;

pub struct ErrorCard<'a, Msg: 'static> {
    title: &'a str,
    msg: Option<&'a str>,
    buttons: Vec<Button<Msg>>,
}

impl<'a, Msg: 'static> ErrorCard<'a, Msg> {
    pub fn from_title(title: &'a str) -> ErrorCard<'a, Msg> {
        ErrorCard {
            title,
            msg: None,
            buttons: Vec::new(),
        }
    }

    pub fn with_msg(mut self, msg: &'a str) -> ErrorCard<'a, Msg> {
        self.msg = Some(msg);
        self
    }

    pub fn with_buttons(mut self, buttons: Vec<Button<Msg>>) -> ErrorCard<'a, Msg> {
        self.buttons = buttons;
        self
    }

    pub fn cell(self) -> Cell<Msg> {
        let msg_row = match self.msg {
            None => Row::none(),
            Some(msg) => Row::from_cells(
                vec![],
                vec![Textarea::simple(msg.to_string()).cell(vec![
                    Style::Grow,
                    Style::H8,
                    Style::W9,
                ])],
            ),
        };

        let buttons_row = if self.buttons.is_empty() {
            Row::none()
        } else {
            Row::from_cells(
                vec![Style::JustifyEnd],
                self.buttons
                    .into_iter()
                    .map(|button| button.cell())
                    .collect(),
            )
        };

        Card::cell_from_rows(
            vec![Style::G4],
            vec![Row::from_str(self.title), msg_row, buttons_row],
        )
    }
}
