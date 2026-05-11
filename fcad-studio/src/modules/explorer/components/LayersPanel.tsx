import { Layers } from "lucide-react";
import { ScrollArea } from "@/modules/core/components/ui/scroll-area";
import { useTranslation } from "react-i18next";
import { useLayers, LayerDef } from "@/modules/core/contexts/LayerContext";
import { useTheme } from "@/modules/core/contexts/ThemeContext";
import { useEffect } from "preact/hooks";

export function LayersPanel() {
  const { t } = useTranslation();
  const { activeLayer, setActiveLayer, adaptedLayers, isLoading, refreshAdaptedLayers } = useLayers();
  const { currentTheme } = useTheme();

  // Re-fetch adapted colors when the theme changes
  useEffect(() => {
    if (currentTheme.value) {
      refreshAdaptedLayers();
    }
  }, [currentTheme.value?.name]);

  if (isLoading.value) {
    return (
      <div className="flex h-full flex-col">
        <div className="text-muted-foreground flex items-center gap-2 border-b p-3 text-xs font-semibold tracking-wider uppercase">
          <Layers className="h-3 w-3" /> {t("ui.panels.layers")}
        </div>
        <div className="text-muted-foreground text-xs animate-pulse p-3">
          Loading layers...
        </div>
      </div>
    );
  }

  // Use adapted layers (theme-aware colors), fallback to a default "0" layer
  const layers = adaptedLayers.value.length > 0 ? adaptedLayers.value : [];
  const activeColor = adaptedLayers.value.find(l => l.name === activeLayer.value)?.color_hex || currentTheme.value?.foreground || "#FFFFFF";

  return (
    <div className="flex h-full flex-col bg-card">
      <div className="text-muted-foreground flex items-center justify-between border-b p-3 text-xs font-semibold tracking-wider uppercase">
        <div className="flex items-center gap-2">
            <Layers className="h-4 w-4" style={{ color: activeColor }} /> 
            <span>{t("ui.panels.layers")}</span>
        </div>
      </div>
      <ScrollArea className="flex-1 p-2">
        <div className="space-y-1">
          {layers.map((layer: LayerDef) => (
            <div
              key={layer.name}
              onClick={() => setActiveLayer(layer.name)}
              className={`flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 text-sm transition-colors ${
                activeLayer.value === layer.name ? "bg-accent/80 text-accent-foreground font-medium border-l-2 border-primary" : "hover:bg-accent/40 border-l-2 border-transparent"
              }`}
            >
              <div
                className="h-3 w-3 rounded-full shrink-0 border border-white/10"
                style={{ backgroundColor: layer.color_hex }}
              />
              <span className="flex-1 truncate">{layer.name}</span>
              {activeLayer.value === layer.name && (
                  <span className="text-[9px] uppercase font-bold tracking-widest text-muted-foreground">Active</span>
              )}
            </div>
          ))}
          {/* Default layer "0" always present */}
          <div 
             onClick={() => setActiveLayer("0")}
             className={`flex cursor-pointer items-center gap-2 rounded px-2 py-1.5 text-sm transition-colors ${
                activeLayer.value === "0" ? "bg-accent/80 text-accent-foreground font-medium border-l-2 border-primary" : "hover:bg-accent/40 border-l-2 border-transparent"
              }`}
          >
            <div
              className="h-3 w-3 rounded-full shrink-0 border border-white/10"
              style={{ backgroundColor: currentTheme.value?.foreground || "#FFFFFF" }}
            />
            <span className="flex-1 truncate">0</span>
            {activeLayer.value === "0" && (
                  <span className="text-[9px] uppercase font-bold tracking-widest text-muted-foreground">Active</span>
            )}
          </div>
        </div>
      </ScrollArea>
    </div>
  );
}
