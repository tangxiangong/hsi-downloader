use ratatui::style::Color;
use yushi_core::config::AppTheme;

/// Color palette for a single theme variant.
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

    // Selection / overlay
    pub selection_bg: Color,
    pub overlay_bg: Color,

    // Text
    pub text: Color,
    pub text_secondary: Color,
    pub text_help: Color,

    // Background
    pub bg: Color,
}

impl ThemeColors {
    /// Dark palette.
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,

            border: Color::Gray,
            border_active: Color::Cyan,

            selection_bg: Color::DarkGray,
            overlay_bg: Color::DarkGray,

            text: Color::White,
            text_secondary: Color::Gray,
            text_help: Color::DarkGray,

            bg: Color::Reset,
        }
    }

    /// Light palette.
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::Gray,

            border: Color::Gray,
            border_active: Color::Blue,

            // Indexed(254) is a near-white gray — readable against a white terminal bg
            selection_bg: Color::Indexed(254),
            overlay_bg: Color::Indexed(250),

            text: Color::Black,
            text_secondary: Color::DarkGray,
            text_help: Color::Gray,

            bg: Color::Reset,
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

                if is_light { Self::light() } else { Self::dark() }
            }
        }
    }
}
