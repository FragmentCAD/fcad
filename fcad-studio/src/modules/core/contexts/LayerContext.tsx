import { createContext } from 'preact';
import { useContext, useEffect } from 'preact/hooks';
import { signal } from '@preact/signals';
import { invoke } from '@tauri-apps/api/core';

export interface LayerDef {
    name: string;
    description: string;
    color_hex: string;
    line_weight: number;
    line_type: string;
}

const activeLayer = signal<string>('0');
const availableLayers = signal<LayerDef[]>([]);
/** Layers with color_hex adapted for the current theme */
const adaptedLayers = signal<LayerDef[]>([]);
const isLoading = signal<boolean>(true);

/** Fetches layers with colors adapted for the active theme */
async function refreshAdaptedLayers() {
    try {
        const layers = await invoke<LayerDef[]>('get_adapted_layers');
        adaptedLayers.value = layers;
    } catch (error) {
        console.error('Failed to load adapted layers:', error);
    }
}

const LayerContext = createContext({
    activeLayer,
    availableLayers,
    adaptedLayers,
    isLoading,
    setActiveLayer: async (name: string) => {
        const result = await invoke<string>('set_active_layer', { name });
        activeLayer.value = result;
    },
    refreshLayers: async () => {
        isLoading.value = true;
        try {
            const layers = await invoke<LayerDef[]>('get_layers');
            availableLayers.value = layers;
            // Also fetch theme-adapted colors
            await refreshAdaptedLayers();
        } catch (error) {
            console.error('Failed to load layers:', error);
        } finally {
            isLoading.value = false;
        }
    },
    refreshAdaptedLayers,
});

export const LayerProvider = ({ children }: { children: any }) => {
    const ctx = useContext(LayerContext);

    useEffect(() => {
        ctx.refreshLayers();
    }, []);

    return (
        <LayerContext.Provider value={ctx}>
            {children}
        </LayerContext.Provider>
    );
};

export const useLayers = () => useContext(LayerContext);
