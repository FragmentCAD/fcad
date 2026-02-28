# FragmentCAD Studio 🎨

Este repositorio contiene **El Cuerpo** de FragmentCAD: la interfaz gráfica de usuario (GUI) construida con Tauri, Vite, Preact y Tailwind CSS.

## 🏗️ Arquitectura (Screaming Architecture)

Para mantener la base de código escalable y limpia conforme la interfaz del CAD crezca, utilizamos una estructura modular conocida como **Screaming Architecture**. En lugar de organizar por tipo de archivo (components, hooks, utils), agrupamos por **funcionalidades o dominios de negocio (features)**.

Al mirar el directorio `src/modules`, la aplicación "grita" de qué se trata:

```text
src/
└── modules/
    ├── ai-console/    # Consola interactiva y chat con el MCP (Model Context Protocol)
    ├── core/          # Componentes de layout general (Header, StatusBar) y utilidades core
    ├── explorer/      # Sistema de capas y gestión de activos/bloques
    ├── properties/    # Panel lateral derecho con propiedades de la entidad seleccionada
    └── viewport/      # Contenedor puente para el renderizado nativo de WGPU (Canvas)
```

## 🛠 Entorno de Desarrollo

El proyecto emplea las siguientes herramientas modernas:

- **Bun** como motor de ejecución JS/TS y gestor de paquetes de alta velocidad.
- **Vite** para HMR (Hot Module Replacement) instantáneo.
- **Preact** por su ligereza y compatibilidad con el ecosistema de React.
- **Tailwind CSS + shadcn/ui:** Mantenemos la filosofía de NO reinventar componentes comunes (modales, popovers, botones). Importamos los componentes base de shadcn y los personalizamos con utilidades de Tailwind.
- **Prettier** con `prettier-plugin-tailwindcss` para mantener todo el código formato.

### Prerequisitos

```bash
# Instalar Tauri CLI (solo la primera vez)
cargo install tauri-cli --version "^2"
```

### Scripts Disponibles

Asegurate de situarte en el directorio `/fcad-studio`:

- **`cargo tauri dev`** ⭐: Levanta la aplicación completa (Frontend Vite + Backend Rust/WGPU). **Este es el comando principal para desarrollo.**
- `bun run build`: Compila los assets del frontend (TypeScript check + Vite bundle).
- `bun run format`: Aplica y auto-corrige el estilo de todos los archivos TypeScript y CSS vía Prettier.
- `bun dev`: Levanta **solo** el frontend Vite (sin backend Rust). Útil para iterar rápido en la UI.

