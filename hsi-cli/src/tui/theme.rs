use hsi_core::config::AppTheme;
use ratatui::style::Color;

/// Color palette for a single theme variant.
///
/// Colors are inspired by the hsi.png mascot: warm peach, coral,
/// cream and cocoa tones.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    // Semantic accent colors
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,

    // Borders
    pub border: Color,
    pub border_active: Color,

    // Selection
    pub selection_bg: Color,

    // Text
    pub text: Color,
    pub text_secondary: Color,
    pub text_help: Color,

    // Background
    pub bg: Color,
}

impl ThemeColors {
    /// Dark palette — warm cocoa background with coral accents.
    pub fn dark() -> Self {
        Self {
            // Coral / salmon — drawn from the warm speckles on the mascot
            primary: Color::Rgb(220, 120, 90),
            // Warm sage green
            success: Color::Rgb(130, 190, 130),
            // Warm amber / honey
            warning: Color::Rgb(220, 180, 90),
            // Warm red-brown
            error: Color::Rgb(210, 95, 85),
            // Muted warm gray
            muted: Color::Rgb(110, 100, 95),

            // Warm brown borders
            border: Color::Rgb(90, 78, 70),
            // Coral accent border
            border_active: Color::Rgb(220, 120, 90),

            // Dark warm brown selection
            selection_bg: Color::Rgb(60, 50, 45),

            // Warm cream text
            text: Color::Rgb(235, 225, 215),
            // Muted peach text
            text_secondary: Color::Rgb(165, 150, 140),
            // Dim warm gray
            text_help: Color::Rgb(110, 100, 95),

            // Deep warm cocoa
            bg: Color::Rgb(35, 28, 25),
        }
    }

    /// Light palette — warm cream background with coral accents.
    pub fn light() -> Self {
        Self {
            // Coral / salmon primary
            primary: Color::Rgb(200, 95, 65),
            // Warm olive green
            success: Color::Rgb(95, 160, 95),
            // Warm amber
            warning: Color::Rgb(200, 155, 55),
            // Warm red
            error: Color::Rgb(195, 75, 65),
            // Muted warm gray
            muted: Color::Rgb(165, 155, 145),

            // Warm peach borders
            border: Color::Rgb(210, 195, 180),
            // Coral accent border
            border_active: Color::Rgb(200, 95, 65),

            // Light peach selection
            selection_bg: Color::Rgb(245, 230, 218),

            // Warm dark brown text
            text: Color::Rgb(55, 40, 30),
            // Medium warm brown
            text_secondary: Color::Rgb(120, 105, 95),
            // Light warm gray
            text_help: Color::Rgb(165, 155, 145),

            // Warm cream
            bg: Color::Rgb(252, 245, 238),
        }
    }

    /// Construct from an `AppTheme`, resolving `System` via the `COLORFGBG`
    /// environment variable (set by most dark-background terminals). Falls
    /// back to dark when the variable is absent or ambiguous.
    pub fn from_app_theme(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Light => Self::light(),
            AppTheme::Dark => Self::dark(),
            AppTheme::System => {
                // `COLORFGBG` is typically "15;0" for dark backgrounds and
                // "0;15" for light backgrounds.  We only treat it as light
                // when the background component (last number) is >= 8.
                let is_light = std::env::var("COLORFGBG")
                    .ok()
                    .and_then(|v| {
                        v.rsplit(';')
                            .next()
                            .and_then(|bg| bg.trim().parse::<u8>().ok())
                    })
                    .map(|bg| bg >= 8)
                    .unwrap_or(false);

                if is_light {
                    Self::light()
                } else {
                    Self::dark()
                }
            }
        }
    }
}
