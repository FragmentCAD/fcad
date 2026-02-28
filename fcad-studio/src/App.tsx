import { useState } from "preact/hooks";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/modules/core/components/ui/resizable";

import { Header } from "@/modules/core/components/layout/Header";
import { StatusBar } from "@/modules/core/components/layout/StatusBar";
import { CanvasViewport } from "@/modules/viewport/components/CanvasViewport";
import { PropertiesPanel } from "@/modules/properties/components/PropertiesPanel";
import { RightSidebar } from "@/modules/core/components/layout/RightSidebar";

import { ThemeProvider } from "@/modules/core/contexts/ThemeContext";
import { LayerProvider } from "@/modules/core/contexts/LayerContext";

export default function App() {
  const [lastHit, setLastHit] = useState<string[]>([]);

  return (
    <ThemeProvider>
      <LayerProvider>
        <div className="text-foreground flex h-screen flex-col overflow-hidden bg-transparent">
          <Header />

          <ResizablePanelGroup direction="horizontal" className="flex-1">
            <ResizablePanel defaultSize={20} minSize={15}>
              <PropertiesPanel lastHit={lastHit} />
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize={60}>
              <div className="relative flex h-full flex-col bg-transparent">
                <CanvasViewport onHitTested={setLastHit} />
              </div>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize={20} minSize={15}>
              <RightSidebar />
            </ResizablePanel>
          </ResizablePanelGroup>

          <StatusBar />
        </div>
      </LayerProvider>
    </ThemeProvider>
  );
}
