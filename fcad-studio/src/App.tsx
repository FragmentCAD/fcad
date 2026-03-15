import { Header } from "@/modules/core/components/layout/Header";
import { StatusBar } from "@/modules/core/components/layout/StatusBar";
import { WorkspaceLayout } from "@/modules/core/components/layout/WorkspaceLayout";
import { ThemeProvider } from "@/modules/core/contexts/ThemeContext";
import { LayerProvider } from "@/modules/core/contexts/LayerContext";

export default function App() {
  return (
    <ThemeProvider>
      <LayerProvider>
        <div className="text-foreground flex h-screen flex-col overflow-hidden bg-transparent">
          <Header />
          <WorkspaceLayout />
          <StatusBar />
        </div>
      </LayerProvider>
    </ThemeProvider>
  );
}
