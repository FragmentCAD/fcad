# 🤖 Agent Guidelines: fcad-studio (The Body)

Este documento define las reglas de juego estrictas para cualquier Agente de IA que escriba código dentro de este repositorio (`fcad-studio`). 

**Contexto:** Este es el frontend del IDE. Debe ser ligero, reactivo y delegar todo el trabajo pesado al Backend Rust.

---

## 1. Stack Tecnológico (No negociable)

*   **Runtime:** `Bun` (No usar `npm` ni `yarn` para instalar paquetes).
*   **Framework:** `Preact` (No React). Usar `preact/signals` para el estado global y local. Evitar `useState`/`useEffect` si un Signal lo resuelve mejor.
*   **Build System:** `Vite` + `Tauri v2`.
*   **Estilos:** `TailwindCSS v4`. No escribir CSS puro ni módulos CSS. Usar clases utilitarias.
*   **Componentes:** `shadcn/ui` (basado en `@radix-ui` primitivos). **REGLA ESTRICTA:** Está prohibido reinventar la rueda o construir componentes visuales comunes (botones, modales, menús, acordeones) desde cero. Tu deber es importar y personalizar los componentes base de shadcn/ui.

## 2. Arquitectura de Carpetas (Screaming Architecture)

No agrupes por "tipo" (ej. no pongas todos los componentes en `/components`). Agrupa por **Dominio/Funcionalidad** para que la estructura "grite" las capacidades del IDE.

```text
src/
├── modules/
│   ├── core/              # Layout base, header, footer, shell.
│   ├── ai-console/        # El chat y control del agente MCP.
│   ├── viewport/          # El lienzo transparente donde vive WGPU.
│   ├── properties/        # Panel de edición de atributos de entidades.
│   ├── explorer/          # Gestión de capas, archivos y activos (Assets).
│   └── domain-tools/      # Herramientas específicas (Arquitectura, Ingeniería).
├── lib/                   # Utilidades compartidas, tipos globales.
└── styles/                # Configuración global de Tailwind.
```

### Reglas de Módulo:
1.  Cada carpeta en `modules/` debe tener su propio `components/`, `hooks/`, `store/` (Signals) e `index.ts` (API pública).
2.  Un módulo no debe importar *internals* de otro módulo. Solo debe usar lo exportado en `index.ts`.

## 3. Patrones de Código

### Estado (Signals over Hooks)
❌ **Mal (React clásico):**
```tsx
const [count, setCount] = useState(0);
// Provoca re-render de todo el componente
```

✅ **Bien (Preact Signals):**
```tsx
const count = useSignal(0);
// Solo actualiza el nodo de texto en el DOM, cero re-renders
return <div>{count}</div>
```

### Comunicación con Rust (Tauri IPC)
*   Nunca llames a `invoke('comando_rust')` directamente en un componente UI.
*   Crea un archivo `api.ts` dentro del módulo correspondiente que envuelva la llamada y tipe la respuesta.

### Rendimiento Crítico
*   **El Viewport es Sagrado:** El área central de la pantalla es transparente. Nunca pongas un `div` con fondo sólido sobre el área de dibujo, o taparás el motor WGPU.
*   **Eventos de Ratón:** Si necesitas capturar clics en el lienzo, asegúrate de que no estás bloqueando el *Event Loop*.

## 4. Flujo de Trabajo (Bun)

*   Instalar dependencias: `bun add <paquete>`
*   Correr dev server: `bun run tauri dev`
*   Testear: `bun test`
*   **Levantar Ventana (Dev):** Ejecuta `bun run tauri dev` en la terminal. Esto compilará el backend de Rust y lanzará la ventana de la aplicación con *Hot Reload* habilitado para el frontend.

---

**Nota Final para el Agente:** Tu objetivo es construir una UI profesional tipo "VS Code". Prioriza la densidad de información, el contraste y la velocidad de respuesta.
