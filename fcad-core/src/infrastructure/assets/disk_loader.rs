use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::infrastructure::ncs::{LayerStandard, StandardsProvider};

/// Estructura DTO de contenedor que coincide con "layers" en el YAML
#[derive(Deserialize)]
struct LayerCollection {
    layers: Vec<LayerStandard>,
}

/// Proveedor de Estándares NCS que carga configuraciones YAML desde disco
pub struct DiskStandardsProvider {
    layers: HashMap<String, LayerStandard>,
}

impl DiskStandardsProvider {
    /// Inicializa un nuevo proveedor leyendo recursivamente un directorio (ej. `standards/`)
    pub fn new(assets_dir: &str) -> Result<Self, String> {
        let mut provider = Self {
            layers: HashMap::new(),
        };

        provider
            .load_directory(assets_dir)
            .map_err(|e| e.to_string())?;

        Ok(provider)
    }

    /// Lee todos los `.yaml` o `.yml` en el directorio de assets dado
    fn load_directory(&mut self, directory: &str) -> std::io::Result<()> {
        let path = Path::new(directory);
        if !path.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy();
                    if ext_str == "yaml" || ext_str == "yml" {
                        self.load_yaml_file(&path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parsea el contenido YAML y carga la lista `LayerCollection` DTO a la memoria del provider
    fn load_yaml_file(&mut self, path: &Path) {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(collection) = serde_yaml::from_str::<LayerCollection>(&contents) {
                for layer in collection.layers {
                    self.layers.insert(layer.name.clone(), layer);
                }
            }
        }
    }
}

/// Lector y Validador Seguro contra ataques de Path Traversal (ej. ../../etc/passwd)
/// Recibe el directorio raíz de assets permitido y el path relativo/absoluto solicitado.
/// Devuelve la PathBuf si es seguro, o un Error (Result) si viola los límites de sanbox.
pub fn resolve_secure_asset_path(
    base_dir: &Path,
    requested_path: &str,
) -> std::io::Result<PathBuf> {
    let base_canonical = base_dir
        .canonicalize()
        .unwrap_or_else(|_| base_dir.to_path_buf());

    // Canonicalize resuelve todo y falla si no existe, por lo que chequeamos ante prefijos
    // Si queremos permitir paths limpios abstractos podemos iterar components.
    // Usaremos un método manual para asegurar que no escape independientemente de la existencia real
    let mut resolved = base_canonical.clone();

    for component in std::path::Path::new(requested_path).components() {
        match component {
            std::path::Component::ParentDir => {
                // Previene escapar la base
                if resolved == base_canonical {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "Path Traversal Violation: Intento de salir del Sandbox de Assets",
                    ));
                }
                resolved.pop();
            }
            std::path::Component::Normal(c) => {
                resolved.push(c);
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                // Rechazar absolutos directos que ignoran la base
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Path Traversal Violation: Rutas absolutas prohibidas",
                ));
            }
            _ => {} // CurDir ('.') se ignora
        }
    }

    // Doble verificación matemática abstracta
    if !resolved.starts_with(&base_canonical) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Path Traversal Violation: Intento de escapar el Sandbox de Assets",
        ));
    }

    Ok(resolved)
}

impl StandardsProvider for DiskStandardsProvider {
    fn get_layer_standard(&self, layer_name: &str) -> Option<LayerStandard> {
        self.layers.get(layer_name).cloned()
    }

    fn get_all_layer_standards(&self) -> Vec<LayerStandard> {
        self.layers.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_yaml_disk_loader() {
        // 1. Arrange: Create a temporary tests directory with a valid standards yaml.
        let test_dir = "test_standards_dir";
        fs::create_dir_all(test_dir).unwrap();

        let yaml_content = r##"
layers:
  - name: "A-WALL"
    description: "Muros de prueba"
    color_hex: "#111111"
    line_weight: 0.5
    line_type: "Continuous"
"##;

        let file_path = format!("{}/test.yaml", test_dir);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        // 2. Act: Instanciamos el loader que simula a la IA pidiendo datos de fcad-assets
        let provider = DiskStandardsProvider::new(test_dir).expect("Error parsing YAML config");

        // 3. Assert
        let wall = provider
            .get_layer_standard("A-WALL")
            .expect("Layer was not read from YAML file");

        assert_eq!(wall.description, "Muros de prueba");
        assert_eq!(wall.color_hex, "#111111");
        assert_eq!(wall.line_weight, 0.5);

        // Cleanup
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_asset_path_traversal_protection() {
        // Arrange
        let test_sandbox = "test_sandbox_assets";
        fs::create_dir_all(test_sandbox).unwrap();
        let base_path = Path::new(test_sandbox);

        // Act & Assert 1: Acceso Bueno / Válido
        let ok_path = resolve_secure_asset_path(base_path, "puerta_sencilla_90.json");
        assert!(
            ok_path.is_ok(),
            "El cargador de fallado una ruta segura normal"
        );
        assert!(ok_path.unwrap().ends_with("puerta_sencilla_90.json"));

        // Act & Assert 2: Sub-Carpetas Válidas
        let ok_sub = resolve_secure_asset_path(base_path, "blocks/architecture/silla.dxf");
        assert!(ok_sub.is_ok());

        // Act & Assert 3: Intento Path Traversal Relativo (Violación)
        let malicious_rel =
            resolve_secure_asset_path(base_path, "../../../Windows/System32/drivers/etc/hosts");
        assert!(
            malicious_rel.is_err(),
            "Se debió generar Panic Controlado (Violación a Traversal)"
        );

        // Act & Assert 4: Intento Path Traversal Absoluto Unix/Windows (Violación)
        let malicious_abs = resolve_secure_asset_path(base_path, "/etc/passwd");
        assert!(
            malicious_abs.is_err(),
            "Rutal Absoluta Directa no respetó el Sandbox!"
        );

        // Act & Assert 5: Intento Enmascarado (Subiendo y Bajando)
        let malicious_hidden =
            resolve_secure_asset_path(base_path, "blocks/../../../../secret.txt");
        assert!(malicious_hidden.is_err());

        // Cleanup
        fs::remove_dir_all(test_sandbox).unwrap();
    }
}
