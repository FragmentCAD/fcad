import { render } from "preact";
import "./App.css";
import "./modules/core/lib/i18n";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
        // Wait for the backend to signal readiness
        const readyPromise = new Promise<void>((resolve) => {
            listen("app-ready", () => resolve());
            // Timeout fallback: if app-ready never fires, proceed anyway after 10s
            setTimeout(resolve, 10000);
        });

        await readyPromise;

        // Sync theme from backend before any rendering
        const theme = await invoke<Theme>("get_current_theme");
        applyThemeToCSS(theme);

        // Store theme for ThemeContext to pick up
        (window as any).__INITIAL_THEME__ = theme;

    } catch (err) {
        console.error("Bootstrap error:", err);
    }

    // Render the app (theme is now applied to CSS)
    render(<App />, document.getElementById("root")!);

    // Close splashscreen and show main window
    try {
        const splashscreen = await WebviewWindow.getByLabel("splashscreen");
        if (splashscreen) {
            await splashscreen.close();
        }
        const mainWindow = getCurrentWindow();
        await mainWindow.show();
        await mainWindow.setFocus();
    } catch (err) {
        console.error("Failed to close splashscreen:", err);
    }
}

bootstrap();
