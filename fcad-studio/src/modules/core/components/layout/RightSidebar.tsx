import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/modules/core/components/ui/tabs";
import { AIConsole } from "@/modules/ai-console/components/AIConsole";
import { ExplorerSidebar } from "@/modules/explorer/components/ExplorerSidebar";
import { Layers, MessageSquare } from "lucide-react";
import { useTranslation } from "react-i18next";

export function RightSidebar() {
  const { t } = useTranslation();

  return (
    <div className="bg-card flex h-full flex-col border-l">
      <Tabs defaultValue="layers" className="flex h-full flex-col">
        <div className="border-b p-2">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="layers" className="text-xs">
              <Layers className="mr-2 h-3 w-3" /> {t("ui.panels.explorer")}
            </TabsTrigger>
            <TabsTrigger value="ai" className="text-xs">
              <MessageSquare className="mr-2 h-3 w-3" /> {t("ui.panels.ai")}
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent
          value="layers"
          className="m-0 flex-1 overflow-auto data-[state=inactive]:hidden"
        >
          {/* Reutiliza el ExplorerSidebar sin sus bordes laterales que ahora gestiona RightSidebar */}
          <ExplorerSidebar />
        </TabsContent>

        <TabsContent
          value="ai"
          className="m-0 flex flex-1 flex-col overflow-hidden data-[state=inactive]:hidden"
        >
          <AIConsole />
        </TabsContent>
      </Tabs>
    </div>
  );
}
