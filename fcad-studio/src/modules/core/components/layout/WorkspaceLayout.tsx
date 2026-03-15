import { Layout, Model, TabNode } from "flexlayout-react";
import { useState, useCallback } from "preact/hooks";
import { defaultLayoutModel } from "../../config/defaultLayout";

// Components
import { CanvasViewport } from "@/modules/viewport/components/CanvasViewport";
import { PropertiesPanel } from "@/modules/properties/components/PropertiesPanel";
import { LayersPanel } from "@/modules/explorer/components/LayersPanel";
import { AssetsPanel } from "@/modules/explorer/components/AssetsPanel";
import { AIConsole } from "@/modules/ai-console/components/AIConsole";
import { DrawingPalette } from "@/modules/tools/components/DrawingPalette";

export const WorkspaceLayout = () => {
    const [model] = useState(() => Model.fromJson(defaultLayoutModel));
    const [lastHit, setLastHit] = useState<string[]>([]);

    const factory = useCallback((node: TabNode) => {
        const component = node.getComponent();

        switch (component) {
            case "canvas":
                return (
                    <div className="relative flex h-full flex-col bg-transparent">
                        <DrawingPalette />
                        <CanvasViewport onHitTested={setLastHit} />
                    </div>
                );
            case "properties":
                return <PropertiesPanel lastHit={lastHit} />;
            case "layers":
                return <LayersPanel />;
            case "assets":
                return <AssetsPanel />;
            case "ai":
                return <AIConsole />;
            default:
                return null;
        }
    }, [lastHit]);

    return (
        <div className="flex-1 relative overflow-hidden bg-transparent">
            <Layout model={model} factory={factory} />
        </div>
    );
};
