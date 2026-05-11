# Modelo de configuración y estándares

**Decisión:** FragmentCAD debe traer defaults sólidos, pero permitir que usuarios, estudios y proyectos definan estándares propios sin modificar el core. Los datos configurables viven como conocimiento declarativo validado, no como constantes hardcodeadas.

## Jerarquía

```text
DefaultConfig
→ UserConfig
→ WorkspaceConfig
→ ProjectConfig
→ Session Overrides
```

La configuración efectiva se obtiene por merge validado y versionado.

## Separación recomendada

| Paquete | Responsabilidad |
|---------|-----------------|
| `fcad-assets` | Defaults, ejemplos, bloques, estándares declarativos. |
| `fcad-config` futuro | Schemas, validación, merge, migraciones y perfiles. |
| `fcad-core` | Consume configuración validada; no parsea convenciones UI. |
| `fcad-studio` | Edita y presenta configuración; no decide reglas de dominio. |

## Estructura objetivo

```text
fcad-assets/
├── defaults/
│   ├── standards/
│   ├── layers/
│   ├── materials/
│   └── styles/
├── schemas/
├── profiles/
└── examples/
```

## Reglas duras

1. No hardcodear capas, grosores, estilos o materiales en TS/Rust si pertenecen a estándares configurables.
2. Todo YAML/JSON de estándares debe tener schema y versión.
3. Defaults del producto nunca deben bloquear perfiles de usuario/proyecto.
4. El merge debe ser determinístico y auditable.
5. Configuración inválida no entra al Core; se reporta con errores accionables.

## Decisión sobre repo/paquete

Mantener `fcad-assets` como contenido y crear `fcad-config` cuando aparezca lógica real de carga, validación, merge o migración. No mover esto a un repo externo todavía: el monorepo preserva consistencia con Core, Studio y Renderer.

## Checklist

- [ ] ¿El dato configurable vive en assets/config y no hardcodeado?
- [ ] ¿Existe schema/versionado?
- [ ] ¿El usuario puede overridear defaults sin forkear assets?
- [ ] ¿Core consume un modelo ya validado?
