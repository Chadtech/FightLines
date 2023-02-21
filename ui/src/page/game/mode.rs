pub mod moving;

#[derive(Debug, Clone)]
pub enum Mode {
    None,
    MovingUnit(moving::Model),
}
