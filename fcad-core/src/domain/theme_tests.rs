#[cfg(test)]
mod tests {
    use crate::domain::theme::Theme;
    use super::*;

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::midnight();
        let json = serde_json::to_string(&theme).unwrap();
        let deserialized: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(theme, deserialized);
        assert_eq!(deserialized.name, "Midnight");
    }

    #[test]
    fn test_theme_default_is_midnight() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Midnight");
        assert_eq!(theme.background, "#000000");
    }

    #[test]
    fn test_architect_theme_values() {
        let theme = Theme::architect();
        assert_eq!(theme.name, "Architect");
        assert_eq!(theme.background, "#FDFDFB");
    }
}
