#[derive(Clone)]
pub enum FrameCount {
    F1,
    F2,
    F3,
    F4,
}

impl FrameCount {
    pub fn to_str(&self) -> &str {
        match self {
            FrameCount::F1 => "1",
            FrameCount::F2 => "2",
            FrameCount::F3 => "3",
            FrameCount::F4 => "4",
        }
    }

    pub fn succ(&self) -> FrameCount {
        match self {
            FrameCount::F1 => FrameCount::F2,
            FrameCount::F2 => FrameCount::F3,
            FrameCount::F3 => FrameCount::F4,
            FrameCount::F4 => FrameCount::F1,
        }
    }
}

pub const ALL: &[FrameCount] = &[
    FrameCount::F1,
    FrameCount::F2,
    FrameCount::F3,
    FrameCount::F4,
];
