import { createContext } from "preact";
import { useContext, useEffect } from "preact/hooks";
import { signal } from "@preact/signals";
import { invoke } from "@tauri-apps/api/core";

export interface Theme {
    name: string;
    background: string;
    foreground: string;
    primary: string;
    accent: string;
    grid_major: string;
    grid_minor: string;
    selection: string;
}

const currentTheme = signal<Theme | null>(null);
const availableThemes = signal<string[]>([]);

/** Applies a Theme's colors to CSS custom properties and dark/light class */
export function applyThemeToCSS(theme: Theme) {
    const root = document.documentElement;
    root.style.setProperty("--background", theme.background);
    root.style.setProperty("--foreground", theme.foreground);
    root.style.setProperty("--primary", theme.primary);
    root.style.setProperty("--accent", theme.accent);

    // Determine dark/light based on background luminosity
    const hex = theme.background.replace("#", "");
    const r = parseInt(hex.substring(0, 2), 16) / 255;
    const g = parseInt(hex.substring(2, 4), 16) / 255;
    const b = parseInt(hex.substring(4, 6), 16) / 255;
    const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;

    if (luminance < 0.5) {
        root.classList.add("dark");
    } else {
        root.classList.remove("dark");
    }
}

const ThemeContext = createContext({
    currentTheme,
    availableThemes,
    switchTheme: async (_name: string) => { },
});

export function ThemeProvider({ children }: { children: any }) {
    const switchTheme = async (name: string) => {
        try {
            const theme = await invoke<Theme>("switch_theme", { themeName: name });
            currentTheme.value = theme;
            applyThemeToCSS(theme);
        } catch (error) {
            console.error("Failed to switch theme:", error);
        }
    };

    useEffect(() => {
        const init = async () => {
            // Load available themes list
            const list = await invoke<string[]>("get_themes_list");
            availableThemes.value = list;

            // Sync current theme from backend (may already be applied by main.tsx bootstrap)
            try {
                const theme = await invoke<Theme>("get_current_theme");
                currentTheme.value = theme;
                applyThemeToCSS(theme);
            } catch (err) {
                // Fallback: check if main.tsx already stored the initial theme
                const initialTheme = (window as any).__INITIAL_THEME__;
                if (initialTheme) {
                    currentTheme.value = initialTheme;
                }
            }
        };
        init();
    }, []);

    return (
        <ThemeContext.Provider value={{ currentTheme, availableThemes, switchTheme }}>
            {children}
        </ThemeContext.Provider>
    );
}

export const useTheme = () => useContext(ThemeContext);
