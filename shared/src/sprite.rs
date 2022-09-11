use crate::facing_direction::FacingDirection;
use crate::frame_count::FrameCount;

#[derive(Clone)]
pub enum Sprite {
    GrassTile,
    Infantry {
        frame: FrameCount,
        dir: FacingDirection,
    },
}

impl Sprite {
    pub fn html_id(&self) -> String {
        self.to_pieces().join("-")
    }
    pub fn to_pieces(&self) -> Vec<String> {
        match self {
            Sprite::GrassTile => vec!["grass".to_string(), "tile".to_string()],
            Sprite::Infantry { frame, dir } => {
                let mut piece = "infantry".to_string();
                piece.push_str(frame.to_str());
                piece.push_str(dir.to_file_name_str());

                vec![piece]
            }
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
