use bevy_ecs::prelude::*;
use fcad_core::domain::architecture::walls;
use fcad_core::domain::{Geometry, ProjectMetadata};
use fcad_core::infrastructure::ecs::spatial::{sync_spatial_index_system, SpatialIndex};
use fcad_core::mcp::server::{McpCommand, McpServer};
use std::env;
use tokio::sync::mpsc;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CRÍTICO: Configurar logs para que salgan estrictamente por stderr.
    // Esto evita contaminar el flujo JSON-RPC que el MCP emite por stdout.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "serve" {
        info!("FragmentCAD MCP Server starting...");

        // Inicializar ECS
        let mut world = World::new();
        world.insert_resource(SpatialIndex::new());
        world.insert_resource(ProjectMetadata::new("Untitled.fcad"));

        // Canal para comunicar el servidor MCP (hilo asíncrono) con el ECS (hilo principal)
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<McpCommand>(100);

        // Iniciar Servidor MCP en un hilo de Tokio
        let mut server = McpServer::new(cmd_tx);
        tokio::spawn(async move {
            if let Err(e) = server.run().await {
                error!("MCP Server error: {}", e);
            }
        });

        // Bucle principal del motor (Simulado para el MVP)
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_spatial_index_system);

        info!("Engine loop active. Waiting for AI commands via MCP...");

        loop {
            // 1. Procesar comandos del canal MPSC (Tarea 4.2 Crítica)
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    McpCommand::GenerateWall {
                        p1,
                        p2,
                        thickness,
                        layer,
                    } => {
                        info!("MCP: Generating wall from {:?} to {:?}", p1, p2);
                        walls::generate_wall(&mut world, p1, p2, thickness, &layer);
                    }
                    McpCommand::ListEntities { resp } => {
                        let count = world.query::<&Geometry>().iter(&world).count();
                        let _ = resp.send(count);
                    }
                }
            }

            // 2. Ejecutar sistemas (Sync de R-Tree, etc.)
            schedule.run(&mut world);

            // 3. Control de FPS (MVP: 10ms por tick)
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    } else {
        println!("FragmentCAD Core Engine");
        println!("Usage: fcad-core serve");
    }

    Ok(())
}
