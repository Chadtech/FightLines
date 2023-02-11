use shared::arrow::Arrow;
use shared::direction::Direction;
use shared::located::Located;
use shared::unit::UnitId;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Model {
    pub unit_id: UnitId,
    pub mobility: HashSet<Located<()>>,
    pub arrows: Vec<(Direction, Arrow)>,
    pub ride_options: Option<Located<RideOptionsModel>>,
}

impl Model {
    pub fn init(unit_id: UnitId, mobility: HashSet<Located<()>>) -> Model {
        Model {
            unit_id,
            mobility,
            arrows: Vec::new(),
            ride_options: None,
        }
    }

    pub fn with_options(
        &mut self,
        x: u16,
        y: u16,
        options: Vec<RideOption>,
        path: Vec<Located<Direction>>,
    ) -> &mut Model {
        let options_model = RideOptionsModel {
            ride_options: options,
            path,
        };

        self.ride_options = Some(Located {
            x,
            y,
            value: options_model,
        });

        self
    }
}

#[derive(Debug)]
pub struct RideOptionsModel {
    pub ride_options: Vec<RideOption>,
    pub path: Vec<Located<Direction>>,
}

#[derive(Debug)]
pub struct RideOption {
    unit_id: UnitId,
    label: String,
}

impl RideOption {
    pub fn init(unit_id: UnitId, label: String) -> RideOption {
        RideOption { unit_id, label }
    }
}
