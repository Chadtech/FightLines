pub struct Time {
    pub day_num: u32,
    pub day_part: TimeOfDay,
}

impl Time {
    pub fn from_turn(turn: u32) -> Result<Time, String> {
        // We want games to start in the morning but also at turn 0.
        // So when it comes to representing the time we shift the turn
        // by 2.
        let shifted_number = turn + 2;

        let day_part = match shifted_number % 6 {
            0 => TimeOfDay::Midnight,
            1 => TimeOfDay::EarlyMorning,
            2 => TimeOfDay::Morning,
            3 => TimeOfDay::Midday,
            4 => TimeOfDay::Evening,
            5 => TimeOfDay::Midnight,
            _ => {
                return Err("turn mod 6 is somehow not less than 6".to_string());
            }
        };

        Ok(Time {
            day_num: shifted_number / 6,
            day_part,
        })
    }
}

impl ToString for Time {
    fn to_string(&self) -> String {
        let mut str = self.day_part.to_string();
        str.push_str(" of day ");
        str.push_str(self.day_num.to_string().as_str());

        str
    }
}

pub enum TimeOfDay {
    Morning,
    Midday,
    Evening,
    Night,
    Midnight,
    EarlyMorning,
}

impl ToString for TimeOfDay {
    fn to_string(&self) -> String {
        match self {
            TimeOfDay::Morning => "morning".to_string(),
            TimeOfDay::Midday => "midday".to_string(),
            TimeOfDay::Evening => "evening".to_string(),
            TimeOfDay::Night => "night".to_string(),
            TimeOfDay::Midnight => "midnight".to_string(),
            TimeOfDay::EarlyMorning => "early morning".to_string(),
        }
    }
}
