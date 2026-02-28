import { ScrollArea } from "@/modules/core/components/ui/scroll-area";
import { useTranslation } from "react-i18next";

interface PropertiesPanelProps {
  lastHit: string[];
}

export function PropertiesPanel({ lastHit }: PropertiesPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-card flex h-full flex-col border-r">
      <div className="text-muted-foreground flex items-center gap-2 border-b p-3 text-xs font-semibold tracking-wider uppercase">
        {t("ui.panels.properties")}
      </div>
      <ScrollArea className="flex-1 p-4">
        {lastHit.length > 0 ? (
          <div className="space-y-4">
            <div className="bg-accent/50 border-accent rounded border p-2">
              <p className="text-muted-foreground text-[10px] font-bold uppercase">
                Selección
              </p>
              <p className="text-primary mt-1 font-mono text-sm">
                {lastHit[0]}
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground">Tipo:</span>
                <span>Línea</span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground">Capa:</span>
                <span className="text-blue-500">A-WALL</span>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex h-full items-center justify-center p-8 text-center">
            <p className="text-muted-foreground text-xs italic">
              {t("ui.panels.properties_empty")}
            </p>
          </div>
        )}
      </ScrollArea>
    </div>
  );
}
