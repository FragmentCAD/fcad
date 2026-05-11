use crate::domain::math::primitives::Point2D;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot};

/// Comandos enviados desde el servidor MCP hacia el motor ECS.
#[derive(Debug)]
pub enum McpCommand {
    GenerateWall {
        p1: Point2D,
        p2: Point2D,
        thickness: f64,
        layer: String,
    },
    ListEntities {
        resp: oneshot::Sender<usize>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    pub id: Option<Value>,
}

pub struct McpServer {
    /// Canal para enviar comandos al hilo principal del ECS.
    pub cmd_tx: mpsc::Sender<McpCommand>,
}

impl McpServer {
    pub fn new(cmd_tx: mpsc::Sender<McpCommand>) -> Self {
        Self { cmd_tx }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        let mut stdout = io::stdout();

        while let Some(line) = reader.next_line().await? {
            let response = self.handle_line(&line).await;
            if let Some(res) = response {
                let json = serde_json::to_string(&res)? + "\n";
                tracing::debug!("Sending response: {}", json);
                // CRÍTICO: stdout solo emite JSON. Logs deben ir a stderr.
                stdout.write_all(json.as_bytes()).await?;
                stdout.flush().await?;
            }
        }
        Ok(())
    }

    async fn handle_line(&mut self, line: &str) -> Option<JsonRpcResponse> {
        tracing::debug!("Received line: {}", line);
        let req: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to parse JSON-RPC: {} | Line: {}", e, line);
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(json!({ "code": -32700, "message": "Parse error" })),
                    id: None,
                });
            }
        };

        match req.method.as_str() {
            "initialize" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "fcad-core", "version": "0.1.0" }
                })),
                error: None,
                id: req.id,
            }),
            "notifications/initialized" => None,
            "tools/list" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({
                    "tools": [
                        {
                            "name": "generar_entidad_parametrica",
                            "description": "Genera geometría compleja (muros, rectángulos) en el motor",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "tipo": { "type": "string", "enum": ["muro", "cuadrado"] },
                                    "parametros": { "type": "object" },
                                    "layer": { "type": "string" }
                                },
                                "required": ["tipo", "parametros"]
                            }
                        },
                        {
                            "name": "obtener_contexto_actual",
                            "description": "Retorna el prompt de sistema ajustado a la disciplina del proyecto actual",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                        {
                            "name": "listar_entidades",
                            "description": "Retorna un resumen de todas las entidades en el motor",
                            "inputSchema": { "type": "object", "properties": {} }
                        }
                    ]
                })),
                error: None,
                id: req.id,
            }),
            "tools/call" => {
                let result = self.handle_tool_call(req.params.as_ref()).await;
                let is_err = result.is_none();
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result,
                    error: if is_err {
                        Some(json!({ "code": -32603, "message": "Internal error" }))
                    } else {
                        None
                    },
                    id: req.id,
                })
            }
            "ping" => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!("pong")),
                error: None,
                id: req.id,
            }),
            _ => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(
                    json!({ "code": -32601, "message": format!("Method not found: {}", req.method) }),
                ),
                id: req.id,
            }),
        }
    }

    async fn handle_tool_call(&mut self, params: Option<&Value>) -> Option<Value> {
        let params = params?;
        let name = params.get("name")?.as_str()?;
        let arguments = params.get("arguments")?;

        // Seguridad: Lista Blanca de herramientas permitidas
        let allowed_tools = [
            "generar_entidad_parametrica",
            "obtener_contexto_actual",
            "listar_entidades",
        ];
        if !allowed_tools.contains(&name) {
            return Some(json!({
                "isError": true,
                "content": [{ "type": "text", "text": format!("Error: Herramienta '{}' no encontrada o acceso denegado.", name) }]
            }));
        }

        match name {
            "generar_entidad_parametrica" => {
                let tipo = arguments.get("tipo")?.as_str()?;
                let layer = arguments
                    .get("layer")
                    .and_then(|l| l.as_str())
                    .unwrap_or("0");

                if tipo == "muro" || tipo == "cuadrado" {
                    let p1_raw = arguments.get("parametros")?.get("p1")?;
                    let p2_raw = arguments.get("parametros")?.get("p2")?;
                    let thickness = arguments
                        .get("parametros")?
                        .get("thickness")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0);

                    let p1 = Point2D::new(p1_raw[0].as_f64()?, p1_raw[1].as_f64()?);
                    let p2 = Point2D::new(p2_raw[0].as_f64()?, p2_raw[1].as_f64()?);

                    // CRÍTICO: Paso de mensajes al ECS
                    let _ = self
                        .cmd_tx
                        .send(McpCommand::GenerateWall {
                            p1,
                            p2,
                            thickness,
                            layer: layer.to_string(),
                        })
                        .await;

                    return Some(json!({
                        "content": [{ "type": "text", "text": format!("Intención de generar {} recibida.", tipo) }]
                    }));
                }
                None
            }
            "listar_entidades" => {
                let (tx, rx) = oneshot::channel();
                let _ = self
                    .cmd_tx
                    .send(McpCommand::ListEntities { resp: tx })
                    .await;
                if let Ok(count) = rx.await {
                    Some(json!({
                        "content": [{ "type": "text", "text": format!("Total de entidades geométricas: {}", count) }]
                    }))
                } else {
                    None
                }
            }
            "obtener_contexto_actual" => Some(json!({
                "content": [{
                    "type": "text",
                    "text": "Contexto NCS: Arquitectónico (A-)."
                }]
            })),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mpsc_command_sent() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut server = McpServer::new(tx);

        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "generar_entidad_parametrica",
                "arguments": {
                    "tipo": "muro",
                    "parametros": {
                        "p1": [0,0],
                        "p2": [10,0]
                    }
                }
            }
        });

        let line = serde_json::to_string(&request).unwrap();
        server.handle_line(&line).await;

        let cmd = rx.try_recv().unwrap();
        match cmd {
            McpCommand::GenerateWall { p1, p2, .. } => {
                assert_eq!(p1.x, 0.0);
                assert_eq!(p2.x, 10.0);
            }
            _ => panic!("Expected GenerateWall command"),
        }
    }
}
