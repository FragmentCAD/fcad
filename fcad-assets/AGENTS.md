# 🤖 Agent Guidelines: fcad-assets (The Memory)

Este documento define las reglas para la gestión de activos y estándares de FragmentCAD.

**Contexto:** Repositorio de datos estructurados (YAML, JSON, DXF). Es la "única fuente de verdad" para estándares de dibujo y componentes predefinidos.

---

## 1. Arquitectura: Domain-Driven Resource Structure

La organización de los archivos debe reflejar la estructura de dominios del resto del ecosistema FCAD.

### Estructura de Carpetas:
```text
blocks/
├── architecture/       # Bloques DXF de carpintería, mobiliario, etc.
└── structural/         # Perfiles, detalles constructivos.
standards/
├── layers/             # Definición de capas por dominio (YAML).
├── styles/             # Estilos de línea y texto (YAML).
└── materials/          # Propiedades físicas y visuales (YAML).
```

## 2. Reglas de Edición

1.  **Inmutabilidad de Versiones:** No modifiques archivos de estándares existentes sin una propuesta (RFC) en `fcad-meta`.
2.  **Metadatos IA:** Cada bloque DXF debe ir acompañado de un archivo `.json` con el mismo nombre que describa semánticamente el bloque para que la IA sepa cuándo y cómo usarlo.
3.  **Validación:** Antes de subir un YAML, asegúrate de que cumple con el esquema definido.
4.  **Configuración Extensible:** Los defaults no deben bloquear overrides de usuario, workspace o proyecto.
5.  **Sin Hardcode:** Capas, grosores, estilos, materiales y templates configurables no deben duplicarse como constantes en Rust/TS.

### Referencias normativas
* `docs/architecture/configuration-and-standards-model.md`
* `skills/fcad-config-assets/SKILL.md`

---
**Nota para el Agente:** Mantén la memoria limpia y organizada. Eres el bibliotecario de FragmentCAD.
