use serde::{Deserialize, Serialize};

/// Calculates the relative luminance of a hex color string (e.g., "#FF0000")
/// per WCAG 2.0 formula: https://www.w3.org/TR/WCAG20/#relativeluminancedef
pub fn luminance(hex: &str) -> f64 {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;

    let linearize = |c: f64| -> f64 {
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };

    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// Calculates the WCAG contrast ratio between two luminance values.
/// Returns a value between 1.0 and 21.0.
pub fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Converts a hex color to HSL (hue 0-360, saturation 0-1, lightness 0-1)
fn hex_to_hsl(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

    let h = if (max - r).abs() < 1e-10 {
        let mut h = (g - b) / d;
        if g < b { h += 6.0; }
        h
    } else if (max - g).abs() < 1e-10 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}

/// Converts HSL to hex color string
fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let hue_to_rgb = |p: f64, q: f64, mut t: f64| -> f64 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    };

    let (r, g, b) = if s.abs() < 1e-10 {
        (l, l, l)
    } else {
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        (
            hue_to_rgb(p, q, h / 360.0 + 1.0 / 3.0),
            hue_to_rgb(p, q, h / 360.0),
            hue_to_rgb(p, q, h / 360.0 - 1.0 / 3.0),
        )
    };

    format!("#{:02X}{:02X}{:02X}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub accent: String,
    pub grid_major: String,
    pub grid_minor: String,
    pub selection: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self::midnight()
    }
}

impl Theme {
    pub fn midnight() -> Self {
        Self {
            name: "Midnight".to_string(),
            background: "#000000".to_string(),
            foreground: "#F0F0F0".to_string(),
            primary: "#3B82F6".to_string(),
            accent: "#1E293B".to_string(),
            grid_major: "#262626".to_string(),
            grid_minor: "#141414".to_string(),
            selection: "#60A5FA".to_string(),
        }
    }

    pub fn architect() -> Self {
        Self {
            name: "Architect".to_string(),
            background: "#FDFDFB".to_string(),
            foreground: "#1A202C".to_string(),
            primary: "#2D3748".to_string(),
            accent: "#CBD5E0".to_string(),
            grid_major: "#E2E8F0".to_string(),
            grid_minor: "#EDF2F7".to_string(),
            selection: "#4A5568".to_string(),
        }
    }

    /// Adapts a layer color_hex for visibility against this theme's background.
    /// If the contrast ratio between the color and the background is below 3.0 (WCAG AA),
    /// the lightness is inverted in HSL space to ensure visibility.
    pub fn adapt_layer_color(&self, color_hex: &str) -> String {
        let color_lum = luminance(color_hex);
        let bg_lum = luminance(&self.background);
        let ratio = contrast_ratio(color_lum, bg_lum);

        if ratio >= 3.0 {
            // Sufficient contrast — keep original color
            return color_hex.to_string();
        }

        // Invert lightness in HSL space
        let (h, s, l) = hex_to_hsl(color_hex);
        let inverted_l = 1.0 - l;
        hsl_to_hex(h, s, inverted_l)
    }
}
