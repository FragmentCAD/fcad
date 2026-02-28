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

const ThemeContext = createContext({
    currentTheme,
    availableThemes,
    switchTheme: async (name: string) => { },
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

    const applyThemeToCSS = (theme: Theme) => {
        const root = document.documentElement;
        root.style.setProperty("--background", theme.background);
        root.style.setProperty("--foreground", theme.foreground);
        root.style.setProperty("--primary", theme.primary);
        root.style.setProperty("--accent", theme.accent);

        // También actualizamos clases de shadcn si es necesario
        if (theme.name.toLowerCase().includes("midnight") || theme.background.startsWith("#0") || theme.background.startsWith("#1")) {
            root.classList.add("dark");
        } else {
            root.classList.remove("dark");
        }
    };

    useEffect(() => {
        // Cargar lista de temas y tema inicial
        const init = async () => {
            const list = await invoke<string[]>("get_themes_list");
            availableThemes.value = list;

            // El backend ya aplica el tema inicial en el setup, 
            // pero necesitamos sincronizar el estado del frontend.
            // Podríamos llamar a switch_theme con el nombre por defecto o añadir un cmd 'get_current_theme'
            // Por ahora, asumimos que 'midnight' o 'architect' según el scroll
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
