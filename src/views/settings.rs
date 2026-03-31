use yushi_core::AppConfig;

pub fn sanitize_theme(theme: &str) -> String {
    match theme {
        "light" | "dark" | "system" => theme.to_string(),
        _ => AppConfig::default().theme,
    }
}
