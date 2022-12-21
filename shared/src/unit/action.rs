use crate::located::Located;

pub enum Action {
    MovedTo(Located<()>),
}
