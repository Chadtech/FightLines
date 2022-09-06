#[derive(Clone)]
pub enum Sprite {
    GrassTile,
    Infantry,
    InfantryLeft,
}

impl Sprite {
    pub fn html_id(&self) -> String {
        self.to_pieces().join("-")
    }
    pub fn to_pieces(&self) -> &[&str] {
        match self {
            Sprite::GrassTile => &["grass", "tile"],
            Sprite::Infantry => &["infantry"],
            Sprite::InfantryLeft => &["infantry-l"],
        }
    }
    pub fn to_file_name(&self) -> String {
        let mut file = self
            .to_pieces()
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("_");

        file.push_str(".png");

        file
    }
}

pub const ALL_SPRITES: &[Sprite] = &[Sprite::GrassTile];
