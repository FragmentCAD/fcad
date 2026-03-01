import { useEffect, useRef, useState } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";

interface CanvasViewportProps {
  onHitTested: (results: string[]) => void;
}

export function CanvasViewport({ onHitTested }: CanvasViewportProps) {
  const canvasRef = useRef<HTMLDivElement>(null);
  const [cameraInfo, setCameraInfo] = useState({ x: 0, y: 0, zoom: 1 });

  // ── Acumuladores para throttling con rAF ──
  const panAccum = useRef({ dx: 0, dy: 0, dirty: false });
  const toolMoveAccum = useRef({ x: 0, y: 0, dirty: false });
  const isPanning = useRef(false);
  const lastPointer = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const el = canvasRef.current;
    if (!el) return;

    // ── Zoom (Mouse Wheel) ──
    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
      // deltaY negativo = scroll up = zoom in
      const factor = e.deltaY < 0 ? 1.1 : 1 / 1.1;

      // Obtener posición relativa al viewport
      const rect = el.getBoundingClientRect();
      const anchorX = e.clientX - rect.left;
      const anchorY = e.clientY - rect.top;

      invoke("send_camera_zoom", {
        factor,
        anchorX,
        anchorY,
      }).catch(console.error);

      setCameraInfo((prev) => ({ ...prev, zoom: prev.zoom * factor }));
    };

    // ── Pan (Middle Mouse Drag) ──
    const handlePointerDown = (e: PointerEvent) => {
      // Middle button (1) o Space+Left (manejado por keydown)
      if (e.button === 1) {
        e.preventDefault();
        isPanning.current = true;
        lastPointer.current = { x: e.clientX, y: e.clientY };
        el.setPointerCapture(e.pointerId);
      }
    };

    const handlePointerMove = (e: PointerEvent) => {
      const rect = el.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (isPanning.current) {
        const dx = e.clientX - lastPointer.current.x;
        const dy = e.clientY - lastPointer.current.y;
        lastPointer.current = { x: e.clientX, y: e.clientY };

        // Acumular deltas para Paneo
        panAccum.current.dx += dx;
        panAccum.current.dy += dy;
        panAccum.current.dirty = true;
      } else {
        // Acumular posición para Herramientas (goma elástica)
        toolMoveAccum.current.x = x;
        toolMoveAccum.current.y = y;
        toolMoveAccum.current.dirty = true;
      }
    };

    const handlePointerUp = (e: PointerEvent) => {
      if (e.button === 1) {
        isPanning.current = false;
        el.releasePointerCapture(e.pointerId);
      }
    };

    // ── Click (Tool Input) ──
    const handleClick = async (e: MouseEvent) => {
      // No procesar si estamos paneando
      if (isPanning.current) return;

      const rect = el.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      const button = e.button === 2 ? "right" : "left";
      try {
        const result = await invoke<string>("send_tool_click", {
          button,
          x,
          y,
        });
        console.log("[ToolManager]", result);
        // Notify parent with tool response
        onHitTested([result]);
      } catch (err) {
        console.error("[ToolManager Error]", err);
      }
    };

    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      // Enviar como clic derecho al ToolManager
      const rect = el.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      invoke<string>("send_tool_click", { button: "right", x, y }).catch(
        console.error,
      );
    };

    // ── rAF Loop: Diferencia entre Pan y ToolMove ──
    let rafId: number;
    const flushInputs = () => {
      // 1. Flush Pan
      if (panAccum.current.dirty) {
        const { dx, dy } = panAccum.current;
        invoke("send_camera_pan", { dx, dy }).catch(console.error);
        setCameraInfo((prev) => ({ ...prev, x: prev.x - dx, y: prev.y + dy }));
        panAccum.current.dx = 0;
        panAccum.current.dy = 0;
        panAccum.current.dirty = false;
      }

      // 2. Flush Tool Move
      if (toolMoveAccum.current.dirty) {
        const { x, y } = toolMoveAccum.current;
        invoke("send_tool_move", { x, y }).catch(console.error);
        toolMoveAccum.current.dirty = false;
      }

      rafId = requestAnimationFrame(flushInputs);
    };
    rafId = requestAnimationFrame(flushInputs);

    // ── Event Listeners ──
    el.addEventListener("wheel", handleWheel, { passive: false });
    el.addEventListener("pointerdown", handlePointerDown);
    el.addEventListener("pointermove", handlePointerMove);
    el.addEventListener("pointerup", handlePointerUp);
    el.addEventListener("click", handleClick);
    el.addEventListener("contextmenu", handleContextMenu);

    // ── Keyboard Shortcuts (F3: Osnap, F7: Grid, F8: Ortho) ──
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "F3") {
        e.preventDefault();
        invoke("toggle_osnaps").catch(console.error);
      } else if (e.key === "F7") {
        e.preventDefault();
        invoke("toggle_grid_snap").catch(console.error);
      } else if (e.key === "F8") {
        e.preventDefault();
        invoke("toggle_ortho").catch(console.error);
      }
    };
    window.addEventListener("keydown", handleKeyDown);

    return () => {
      cancelAnimationFrame(rafId);
      el.removeEventListener("wheel", handleWheel);
      el.removeEventListener("pointerdown", handlePointerDown);
      el.removeEventListener("pointermove", handlePointerMove);
      el.removeEventListener("pointerup", handlePointerUp);
      el.removeEventListener("click", handleClick);
      el.removeEventListener("contextmenu", handleContextMenu);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  // ── ResizeObserver (sincroniza dimensiones con WGPU) ──
  useEffect(() => {
    const observer = new ResizeObserver((entries) => {
      for (let entry of entries) {
        const rect = entry.target.getBoundingClientRect();
        invoke("update_viewport_rect", {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        }).catch((err) =>
          console.error("Error sincronizando ventana WGPU:", err),
        );
      }
    });

    if (canvasRef.current) {
      observer.observe(canvasRef.current);
    }

    return () => observer.disconnect();
  }, []);

  return (
    <div
      id="viewport-canvas"
      ref={canvasRef}
      className="relative flex flex-1 cursor-crosshair items-center justify-center bg-transparent"
      style={{ touchAction: "none" }}
    >
      {/* Este es el hueco donde WGPU renderiza nativamente */}


      {/* HUD / Overlay — Info de Cámara en Tiempo Real */}
      <div className="absolute top-4 left-4 rounded border border-white/10 bg-black/50 p-2 font-mono text-[10px] text-white/70">
        X: {cameraInfo.x.toFixed(1)} Y: {cameraInfo.y.toFixed(1)} | Z:{" "}
        {cameraInfo.zoom.toFixed(2)}x
      </div>
    </div>
  );
}
