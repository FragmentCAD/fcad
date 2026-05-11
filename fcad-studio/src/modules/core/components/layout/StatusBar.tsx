import { useTranslation } from "react-i18next";
import { useEffect, useState } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { MoveRight, Target, Grid3X3 } from "lucide-react";

interface SnapState {
  ortho_enabled: boolean;
  osnaps_enabled: boolean;
  grid_snap_enabled: boolean;
  grid_size: number;
}

export function StatusBar() {
  const { t } = useTranslation();
  const [snapState, setSnapState] = useState<SnapState | null>(null);

  useEffect(() => {
    invoke<SnapState>("get_snap_state").then(setSnapState).catch(console.error);
  }, []);

  const toggle = async (cmd: string) => {
    await invoke<boolean>(cmd);
    // Refresh the whole state for simplicity
    invoke<SnapState>("get_snap_state").then(setSnapState).catch(console.error);
  };

  return (
    <footer className="bg-background text-muted-foreground flex h-8 items-center justify-between border-t px-4 text-[11px] font-medium shadow-[0_-2px_10px_rgba(0,0,0,0.1)]">
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2 pr-4 border-r border-border/50">
          <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          <span className="uppercase tracking-wider opacity-80">{t("ui.statusbar.ready")}</span>
        </div>

        <div className="flex gap-1">
          <button
            onClick={() => toggle("toggle_ortho")}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded transition-all hover:bg-accent ${snapState?.ortho_enabled ? 'bg-primary/15 text-primary' : 'opacity-60'}`}
            title="Ortho Mode (F8)"
          >
            <MoveRight size={14} />
            <span className="hidden sm:inline">ORTHO</span>
          </button>

          <button
            onClick={() => toggle("toggle_osnaps")}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded transition-all hover:bg-accent ${snapState?.osnaps_enabled ? 'bg-primary/15 text-primary' : 'opacity-60'}`}
            title="Osnap (F3)"
          >
            <Target size={14} />
            <span className="hidden sm:inline">OSNAP</span>
          </button>

          <button
            onClick={() => toggle("toggle_grid_snap")}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded transition-all hover:bg-accent ${snapState?.grid_snap_enabled ? 'bg-primary/15 text-primary' : 'opacity-60'}`}
            title="Grid Snap (F7)"
          >
            <Grid3X3 size={14} />
            <span className="hidden sm:inline">GRID</span>
          </button>
        </div>
      </div>

      <div className="flex items-center gap-4 opacity-70">
        <div className="flex gap-2">
          <span>FPS: 144</span>
          <span className="text-border">|</span>
          <span>{t("ui.statusbar.tech_stack")}</span>
        </div>
      </div>
    </footer>
  );
}
