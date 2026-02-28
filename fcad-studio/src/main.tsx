import { render } from "preact";
import "./App.css";
import "./modules/core/lib/i18n";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

interface Theme {
    name: string;
    background: string;
    foreground: string;
    primary: string;
    accent: string;
    grid_major: string;
    grid_minor: string;
    selection: string;
}

function applyThemeToCSS(theme: Theme) {
    const root = document.documentElement;
    root.style.setProperty("--background", theme.background);
    root.style.setProperty("--foreground", theme.foreground);
    root.style.setProperty("--primary", theme.primary);
    root.style.setProperty("--accent", theme.accent);

    // Apply dark mode class based on background luminosity
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

async function bootstrap() {
    try {
        // Sync theme from backend — this command is always available once Tauri is ready
        const theme = await invoke<Theme>("get_current_theme");
        applyThemeToCSS(theme);

        // Store theme for ThemeContext to pick up
        (window as any).__INITIAL_THEME__ = theme;
    } catch (err) {
        console.error("Bootstrap: failed to get theme, using defaults:", err);
    }

    // Render the app (theme is now applied to CSS variables)
    render(<App />, document.getElementById("root")!);

    // Close splashscreen and show main window
    try {
        const splashscreen = await WebviewWindow.getByLabel("splashscreen");
        if (splashscreen) {
            await splashscreen.close();
        }
    } catch (err) {
        // Splashscreen may not exist in some environments
        console.warn("Splashscreen close:", err);
    }

    try {
        const mainWindow = getCurrentWindow();
        await mainWindow.show();
        await mainWindow.setFocus();
    } catch (err) {
        console.error("Failed to show main window:", err);
    }
}

bootstrap();
