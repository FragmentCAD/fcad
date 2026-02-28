import { useLayers, LayerDef } from "@/modules/core/contexts/LayerContext";

export function LayerSelector() {
    const { activeLayer, availableLayers, adaptedLayers, isLoading, setActiveLayer } = useLayers();

    if (isLoading.value) {
        return <div className="text-muted-foreground text-xs animate-pulse px-2">Loading layers...</div>;
    }

    // Use adapted layer color for the indicator dot
    const activeAdaptedColor = adaptedLayers.value.find(l => l.name === activeLayer.value)?.color_hex || '#FFFFFF';

    return (
        <div className="bg-background/50 flex items-center gap-2 rounded-md border p-1 shadow-sm">
            <span className="text-muted-foreground px-2 text-[10px] font-bold uppercase tracking-wider">Layer</span>
            <select
                value={activeLayer.value}
                onChange={(e) => setActiveLayer((e.target as HTMLSelectElement).value)}
                className="bg-transparent text-xs font-medium focus:outline-none cursor-pointer hover:text-primary transition-colors"
            >
                <option value="0">0 (Default)</option>
                {availableLayers.value.map((layer: LayerDef) => (
                    <option key={layer.name} value={layer.name}>
                        {layer.name} — {layer.description}
                    </option>
                ))}
            </select>

            {/* Indicador de color de capa (theme-adapted) */}
            <div
                className="h-3 w-3 rounded-full border border-white/10"
                style={{ backgroundColor: activeAdaptedColor }}
            />
        </div>
    );
}
