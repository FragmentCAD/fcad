import { Layers } from "lucide-react";
import { ScrollArea } from "@/modules/core/components/ui/scroll-area";
import { useTranslation } from "react-i18next";
import { useLayers, LayerDef } from "@/modules/core/contexts/LayerContext";
import { useTheme } from "@/modules/core/contexts/ThemeContext";
import { useEffect } from "preact/hooks";

export function LayersPanel() {
  const { t } = useTranslation();
  const { adaptedLayers, isLoading, refreshAdaptedLayers } = useLayers();
  const { currentTheme } = useTheme();

  // Re-fetch adapted colors when the theme changes
  useEffect(() => {
    if (currentTheme.value) {
      refreshAdaptedLayers();
    }
  }, [currentTheme.value?.name]);

  if (isLoading.value) {
    return (
      <>
        <div className="text-muted-foreground flex items-center gap-2 border-b p-3 text-xs font-semibold tracking-wider uppercase">
          <Layers className="h-3 w-3" /> {t("ui.panels.layers")}
        </div>
        <div className="text-muted-foreground text-xs animate-pulse p-3">
          Loading layers...
        </div>
      </>
    );
  }

  // Use adapted layers (theme-aware colors), fallback to a default "0" layer
  const layers = adaptedLayers.value.length > 0 ? adaptedLayers.value : [];

  return (
    <>
      <div className="text-muted-foreground flex items-center gap-2 border-b p-3 text-xs font-semibold tracking-wider uppercase">
        <Layers className="h-3 w-3" /> {t("ui.panels.layers")}
      </div>
      <ScrollArea className="flex-1 p-2">
        <div className="space-y-1">
          {layers.map((layer: LayerDef) => (
            <div
              key={layer.name}
              className="hover:bg-accent flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm"
            >
              <div
                className="h-2 w-2 rounded-full shrink-0"
                style={{ backgroundColor: layer.color_hex }}
              />
              {layer.name}
            </div>
          ))}
          {/* Default layer "0" always present */}
          <div className="hover:bg-accent flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm">
            <div
              className="h-2 w-2 rounded-full shrink-0"
              style={{ backgroundColor: currentTheme.value?.foreground || "#FFFFFF" }}
            />
            0
          </div>
        </div>
      </ScrollArea>
    </>
  );
}
