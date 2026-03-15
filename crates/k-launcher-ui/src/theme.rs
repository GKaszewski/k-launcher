use iced::{
    Color, Gradient,
    gradient::{ColorStop, Linear},
};

pub struct AeroColors {
    pub glass_bg: Color,
    pub gloss_highlight: Gradient,
    pub border_cyan: Color,
}

impl AeroColors {
    pub fn standard() -> Self {
        Self {
            // Semi-transparent "Aero Glass" base
            glass_bg: Color::from_rgba8(255, 255, 255, 0.2),
            // Cyan/Blue glow typical of the 2008 era
            border_cyan: Color::from_rgb8(0, 183, 235),
            // We'll use this for the "shine" effect on buttons
            gloss_highlight: Gradient::Linear(Linear::new(0.0).add_stops([
                ColorStop {
                    color: Color::from_rgba8(255, 255, 255, 0.5),
                    offset: 0.0,
                },
                ColorStop {
                    color: Color::from_rgba8(255, 255, 255, 0.0),
                    offset: 1.0,
                },
            ])),
        }
    }
}
