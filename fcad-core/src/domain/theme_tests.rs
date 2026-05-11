#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme::Theme;

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

    // === TDD: adapt_layer_color tests ===

    #[test]
    fn test_luminance_black() {
        use crate::domain::theme::luminance;
        let l = luminance("#000000");
        assert!(
            (l - 0.0).abs() < 0.001,
            "Black should have luminance ~0.0, got {}",
            l
        );
    }

    #[test]
    fn test_luminance_white() {
        use crate::domain::theme::luminance;
        let l = luminance("#FFFFFF");
        assert!(
            (l - 1.0).abs() < 0.001,
            "White should have luminance ~1.0, got {}",
            l
        );
    }

    #[test]
    fn test_luminance_red() {
        use crate::domain::theme::luminance;
        let l = luminance("#FF0000");
        // sRGB red has relative luminance of ~0.2126
        assert!(
            (l - 0.2126).abs() < 0.01,
            "Red should have luminance ~0.2126, got {}",
            l
        );
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        use crate::domain::theme::contrast_ratio;
        let cr = contrast_ratio(0.0, 1.0);
        assert!(
            (cr - 21.0).abs() < 0.1,
            "Black vs white should be ~21:1, got {}",
            cr
        );
    }

    #[test]
    fn test_adapt_black_on_black_background() {
        // Black layer on black background = invisible → must adapt
        let theme = Theme::midnight(); // background #000000
        let adapted = theme.adapt_layer_color("#000000");
        assert_ne!(
            adapted, "#000000",
            "Black on black must be adapted to avoid invisibility"
        );
    }

    #[test]
    fn test_adapt_red_on_black_background() {
        // Red layer on black background = visible → keep original
        let theme = Theme::midnight(); // background #000000
        let adapted = theme.adapt_layer_color("#FF0000");
        assert_eq!(
            adapted, "#FF0000",
            "Red on black has sufficient contrast, should remain unchanged"
        );
    }

    #[test]
    fn test_adapt_black_on_white_background() {
        // Black layer on white background = visible → keep original
        let theme = Theme::architect(); // background #FDFDFB
        let adapted = theme.adapt_layer_color("#000000");
        assert_eq!(
            adapted, "#000000",
            "Black on white has sufficient contrast, should remain unchanged"
        );
    }

    #[test]
    fn test_adapt_white_on_white_background() {
        // White layer on white background = invisible → must adapt
        let theme = Theme::architect(); // background #FDFDFB
        let adapted = theme.adapt_layer_color("#FDFDFB");
        assert_ne!(
            adapted, "#FDFDFB",
            "White on white must be adapted to avoid invisibility"
        );
    }

    #[test]
    fn test_adapt_cyan_on_black_background() {
        // Cyan (#00FFFF) on black = visible → keep original
        let theme = Theme::midnight();
        let adapted = theme.adapt_layer_color("#00FFFF");
        assert_eq!(
            adapted, "#00FFFF",
            "Cyan on black has sufficient contrast, should remain unchanged"
        );
    }
}
