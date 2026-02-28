import { LayersPanel } from "./LayersPanel";
import { AssetsPanel } from "./AssetsPanel";

export function ExplorerSidebar() {
  return (
    <div className="bg-card flex h-full flex-col border-r">
      <LayersPanel />
      <AssetsPanel />
    </div>
  );
}
