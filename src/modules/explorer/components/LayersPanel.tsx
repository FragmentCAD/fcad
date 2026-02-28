import { Layers } from "lucide-react";
import { ScrollArea } from "@/modules/core/components/ui/scroll-area";
import { useTranslation } from "react-i18next";

export function LayersPanel() {
  const { t } = useTranslation();

  return (
    <>
      <div className="text-muted-foreground flex items-center gap-2 border-b p-3 text-xs font-semibold tracking-wider uppercase">
        <Layers className="h-3 w-3" /> {t("ui.panels.layers")}
      </div>
      <ScrollArea className="flex-1 p-2">
        <div className="space-y-1">
          {["A-WALL", "A-DOOR", "A-WINDOW", "0"].map((layer) => (
            <div
              key={layer}
              className="hover:bg-accent flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm"
            >
              <div className="h-2 w-2 rounded-full bg-blue-500" /> {layer}
            </div>
          ))}
        </div>
      </ScrollArea>
    </>
  );
}
