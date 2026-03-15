import { Layout, Model, TabNode } from "flexlayout-react";
import { useState, useCallback } from "preact/hooks";
import { defaultLayoutModel } from "../../config/defaultLayout";

// Components
import { CanvasViewport } from "@/modules/viewport/components/CanvasViewport";
import { PropertiesPanel } from "@/modules/properties/components/PropertiesPanel";
import { RightSidebar } from "@/modules/core/components/layout/RightSidebar";
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
            case "right-sidebar":
                return <RightSidebar />;
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
