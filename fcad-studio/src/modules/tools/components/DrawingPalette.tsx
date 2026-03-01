import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { MousePointer2, Minus, Square } from "lucide-react";

export function DrawingPalette() {
    const [activeTool, setActiveTool] = useState<string>("none");

    useEffect(() => {
        // Sincronizar estado inicial
        invoke<string>("get_active_tool").then(setActiveTool);
    }, []);

    const handleSetTool = async (toolName: string) => {
        // Si ya es el activo, lo desactivamos (toggle)
        const newTool = activeTool === toolName ? "none" : toolName;
        const result = await invoke<string>("set_active_tool", { toolName: newTool });
        setActiveTool(result);
    };

    const buttons = [
        { id: "erase", label: "Select/Erase", icon: <MousePointer2 size={20} /> },
        { id: "line", label: "Line", icon: <Minus size={20} /> },
        { id: "rect", label: "Rectangle", icon: <Square size={20} /> },
    ];

    return (
        <div className="absolute left-4 top-4 z-50 flex flex-col gap-2 rounded-xl border border-white/10 bg-black/40 p-2 backdrop-blur-md shadow-2xl">
            {buttons.map((btn) => (
                <button
                    key={btn.id}
                    onClick={() => handleSetTool(btn.id)}
                    title={btn.label}
                    className={`flex h-10 w-10 items-center justify-center rounded-lg transition-all duration-200 ${activeTool === btn.id
                            ? "bg-primary text-primary-foreground shadow-lg scale-105"
                            : "text-white/70 hover:bg-white/10 hover:text-white"
                        }`}
                >
                    {btn.icon}
                </button>
            ))}
        </div>
    );
}
