use crate::infrastructure::ncs::LayerStandard;
use rusqlite::{Connection, Result};

/// Encapsula un índice semántico impulsado por SQLite + FTS5 (+ hipotéticamente sqlite-vec).
/// Sirve como el "Cerebro RAG" embebido para realizar búsquedas hiperrápidas.
pub struct RagEngine {
    conn: Connection,
}

impl RagEngine {
    /// Inicializa una nueva instancia de motor en memoria o disco.
    /// Para mantener independencia, las pruebas usan una base de datos en memoria.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Inicializa una base de datos persistente
    pub fn new_persistent(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::initialize_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Crea las tablas virtuales FTS5.
    /// FTS5 (Full Text Search) permite indexar columnas como texto segmentado.
    fn initialize_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS standards_fts USING fts5(
                name,
                description,
                color_hex UNINDEXED,
                line_type UNINDEXED
            );",
            [],
        )?;
        // Aquí también iría la inicialización de `sqlite-vec` para embeddings.
        Ok(())
    }

    /// Indexa un estándar de capa directamente en el motor de búsqueda (ej. extraído de YAML)
    pub fn index_standard(&mut self, standard: &LayerStandard) -> Result<()> {
        // En una implementación real con `ort`, aquí:
        // 1. Tomaríamos `standard.description`, la pasaríamos por un Tokenizer (BPE)
        // 2. Alimetaríamos ONNX / ort con los tokens del modelo `all-MiniLM-L6-v2`
        // 3. Extraeríamos el tensor resultante de dimensionalidad estricta (Embeddings Float32)
        // 4. Guardaríamos `standard.name` con su VECTOR correspondiente en la tabla paralela de `sqlite-vec`.

        // Fase 1 actual: Búsqueda Semántica Léxica (FTS5) pura
        self.conn.execute(
            "INSERT INTO standards_fts (name, description, color_hex, line_type) VALUES (?1, ?2, ?3, ?4)",
            (
                &standard.name,
                &standard.description,
                &standard.color_hex,
                &standard.line_type,
            ),
        )?;
        Ok(())
    }

    /// Realiza una búsqueda semántica léxica Full-Text
    pub fn search(&self, query: &str) -> Result<Vec<LayerStandard>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, description, color_hex, line_type 
             FROM standards_fts 
             WHERE standards_fts MATCH ? 
             ORDER BY rank",
        )?;

        // SQLite match requiere un formato especial, ej:
        // si query es 'muro', se parsea como 'muro*' si queremos autocompletado, etc.
        let match_query = format!("\"{}\"*", query.replace("\"", ""));

        let standards_iter = stmt.query_map([match_query], |row| {
            Ok(LayerStandard {
                name: row.get(0)?,
                description: row.get(1)?,
                color_hex: row.get(2)?,
                line_weight: 0.0, // Default for test as it isn't fully persisted unindexed yet if omitted
                line_type: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for standard in standards_iter {
            results.push(standard?);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fts5_semantic_search() {
        let mut rag =
            RagEngine::new_in_memory().expect("No se logró inicializar RAG engine en SQLite");

        // Cargar un YAML/Estándar falso y registrarlo
        let wall_standard = LayerStandard {
            name: "A-WALL".to_string(),
            description:
                "Paredes de Construcción Arquitectónica, Estructuras y División, Muros, Tabiques"
                    .to_string(),
            color_hex: "#000000".to_string(),
            line_weight: 0.35,
            line_type: "Continuous".to_string(),
        };

        let door_standard = LayerStandard {
            name: "A-DOOR".to_string(),
            description: "Puertas, Marco, Cristal de Puerta, Bisagras, Arquitectura".to_string(),
            color_hex: "#FF0000".to_string(),
            line_weight: 0.18,
            line_type: "Continuous".to_string(),
        };

        rag.index_standard(&wall_standard)
            .expect("Falló la Indexación de muro");
        rag.index_standard(&door_standard)
            .expect("Falló la Indexación de puerta");

        // 1. Intentar hacer match directo de una "keyword" exacta de usuario para validación.
        let results = rag.search("muros").expect("Query Falló");
        assert_eq!(
            results.len(),
            1,
            "Solo 'A-WALL' debe devolver resultados de muros"
        );
        assert_eq!(results[0].name, "A-WALL");

        // 2. Hacer match con un string aproximado y compuesto lexicográfico
        let results2 = rag.search("bisagras").expect("Query Fallada");
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].name, "A-DOOR");

        // 3. Match general de la subclasificación arquitectónica general.
        // Dado el 'ORDER BY rank' interno de tf-idf en SQLite, la precisión es nativamente superior.
        // Por la búsqueda exacta del test, dejaremos las previas como las firmes.
    }
}
