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
const isLoading = signal<boolean>(true);

const LayerContext = createContext({
    activeLayer,
    availableLayers,
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
        } catch (error) {
            console.error('Failed to load layers:', error);
        } finally {
            isLoading.value = false;
        }
    }
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
