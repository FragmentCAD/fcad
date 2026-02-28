
> **Nota de Arquitectura:** Este repositorio es parte del ecosistema FragmentCAD v0.1.0.
> Para entender la visión global, la interacción entre repositorios y las decisiones de diseño (AI-First, ECS, WGPU), visita el repositorio principal: [**fcad-meta**](https://github.com/FragmentCAD/fcad-meta).

---

# `fcad-assets`: El Ecosistema de Datos Abierto y Extensible

**Propósito:** Actuar como la biblioteca central ("La Memoria") de FragmentCAD, proporcionando contexto estandarizado a los agentes de IA y plantillas listas para usar para los usuarios humanos.
**Filosofía:** Todo debe ser texto plano (JSON/YAML) o estándares abiertos (DXF). La extensibilidad debe ser tan sencilla como arrastrar y soltar un archivo o editar un archivo de texto, sin necesidad de compilar código.

---

## 1. El Problema que Resuelve

En el CAD tradicional, los estándares de la empresa (capas, grosores, bloques) suelen estar atrapados en formatos binarios oscuros (`.dwt`, `.lin`, `.pat`) o escondidos en bases de datos internas complejas. Esto presenta dos grandes problemas:
1.  **Fricción para el Usuario:** Es difícil para una empresa o usuario independiente imponer su propio estilo o compartir su biblioteca de bloques de forma versionable (Git).
2.  **Ceguera para la IA:** Un agente inteligente (vía MCP) no puede "leer" un archivo binario `.dwt` para entender qué capa debe usar cuando se le pide "dibujar un muro de carga".

**La Solución de FragmentCAD:** `fcad-assets` expone toda esta "inteligencia de dominio" en archivos legibles, permitiendo que tanto humanos como máquinas entiendan y modifiquen las reglas del juego instantáneamente.

---

## 2. Estructura de Directorios (El "ADN" del Proyecto)

El repositorio se organiza semánticamente para facilitar la indexación automática durante el arranque de la aplicación (`fcad-core` lee esta carpeta al inicializarse):

```text
fcad-assets/
├── blocks/                 # La biblioteca de componentes geométricos
│   ├── architecture/       # Categorías (Carpetas simples)
│   │   ├── furniture/
│   │   │   ├── chair_standard.dxf    # El archivo CAD puro (Geometría estática)
│   │   │   └── chair_standard.json   # El Manifiesto (Metadatos e Inteligencia IA)
│   │   └── doors/              # (Solo JSONs que apuntan a generadores en Rust, sin DXF)
│   └── structural/
├── standards/              # Reglas y Estilos del Proyecto/Empresa
│   ├── layers.yaml         # Definición de capas estándar (Nombres, Colores, Grosores)
│   ├── linetypes.yaml      # Estilos de línea (Continua, Punteada, Ejes)
│   └── text_styles.yaml    # Estilos de texto y cotas
├── templates/              # Archivos de inicio rápido (.dxf base)
│   └── default_arch_metric.dxf
└── i18n/                   # Diccionarios de Traducción (Para UI e IA)
    ├── en.json
    └── es.json
```

---

## 3. El Paradigma de los "Bloques Inteligentes" (DXF + JSON)

Los elementos en FragmentCAD se dividen en **Bloques Estáticos** (muebles, sanitarios) y **Generadores Paramétricos** (muros, puertas).

Los archivos DXF se usan *estrictamente* para símbolos estáticos inmutables. Para convertirlos en herramientas útiles para la IA, usamos el concepto de **Manifiesto Adjunto**.

Por cada archivo de geometría (`ejemplo.dxf`), existe un archivo de metadatos con exactamente el mismo nombre (`ejemplo.json`).

**Ejemplo: `blocks/architecture/furniture/chair_standard.json`**

```json
{
  "id": "arch_chair_standard",
  "name": {
    "en": "Standard Chair",
    "es": "Silla Estándar"
  },
  "description": "Standard office chair.",
  "category": "architecture/furniture",
  "insertion_point": "center_bottom",
  "parameters": {
    "scale": "uniform",
    "material": "fabric"
  },
  "ai_tags": ["chair", "office", "furniture", "silla", "oficina", "mueble"]
}
```

### ¿Por qué esto es revolucionario?
1.  **Contexto IA Inmediato:** El servidor MCP lee este JSON. Cuando un usuario le dice al Agente: *"Inserta una silla de oficina estándar"*, la IA busca en los `ai_tags`, encuentra la coincidencia y sabe exactamente qué bloque insertar y cuáles son sus parámetros (ancho/alto) por defecto.
2.  **Extensibilidad Humana:** Si un arquitecto en Argentina quiere que la IA entienda el término local, solo tiene que abrir el JSON y agregar `"silla_ergonomica"` a los `ai_tags`.

---

## 4. Estándares Abiertos (YAML)

En lugar de forzar a los usuarios a navegar por menús de UI complejos para configurar las capas de su empresa, todo se define en texto plano.

**Ejemplo: `standards/layers_arch_metric.yaml`**

```yaml
standard_name: "AIA_Simplified_Metric"
description: "Estándar simplificado basado en AIA para proyectos métricos."
layers:
  - name: "A-WALL-FULL"
    color: "#FF0000"       # Rojo
    lineweight: 0.50       # mm
    linetype: "Continuous"
    description: "Muros de carga o altura completa."
    ai_context: "Use this layer for main structural walls and full height partitions."
  - name: "A-DOOR"
    color: "#00FF00"       # Verde
    lineweight: 0.25
    linetype: "Continuous"
    description: "Puertas y marcos."
```

### El Flujo de Trabajo (Humano + Máquina)
Cuando el usuario ejecuta `fcad init` para empezar un proyecto nuevo, FragmentCAD lee este archivo YAML e inyecta estas capas en el motor ECS en memoria del proyecto.
A partir de ese momento, la IA consulta el motor RAG de `fcad-agent-skills`: si se le pide *"Dibuja los muros principales"*, el motor busca y extrae la propiedad `ai_context` del YAML y le indica a la IA que debe usar la capa `A-WALL-FULL`.

---

## 5. Localización e i18n

La carpeta `i18n/` contiene diccionarios simples (`es.json`, `en.json`).
Estos archivos no solo traducen los botones de la interfaz de `fcad-studio` (Frontend), sino que también **traducen las intenciones de las herramientas en el servidor MCP** (Backend).

Esto asegura que un Agente de IA pueda explicarle sus acciones a un usuario hispanohablante de manera natural, utilizando la jerga técnica correcta definida en estos diccionarios.

---

## Resumen: El Poder de lo Simple
Al mantener `fcad-assets` como una colección de archivos de texto plano (YAML/JSON) y formatos estándar (DXF):
*   Es 100% versionable con Git.
*   Es el "contexto inyectable" perfecto para los LLMs.
*   Permite a cualquier estudio de arquitectura o ingeniería "forkear" este repositorio, modificar los JSON/YAML a su gusto, y tener instantáneamente un ecosistema CAD personalizado y una IA entrenada en sus propios estándares de empresa.
