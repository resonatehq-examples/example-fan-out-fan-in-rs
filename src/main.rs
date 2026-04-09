use resonate::prelude::*;
use serde::{Deserialize, Serialize};

/// Input for a single work item.
#[derive(Serialize, Deserialize, Clone)]
struct WorkItem {
    id: u32,
    data: String,
}

/// Result from processing a single work item.
#[derive(Serialize, Deserialize, Debug)]
struct WorkResult {
    id: u32,
    output: String,
}

/// Fan-out/fan-in workflow.
///
/// Spawns three tasks in parallel using `.spawn()` with remote invocation,
/// then collects all results through durable handles.
/// If the process crashes mid-execution, only incomplete items re-execute.
#[resonate::function]
async fn fan_out_fan_in(ctx: &Context, items: Vec<WorkItem>) -> Result<Vec<WorkResult>> {
    // Fan-out: spawn all three items in parallel via rpc
    let h1 = ctx.rpc::<WorkResult>("process_item", items[0].clone()).spawn().await?;
    let h2 = ctx.rpc::<WorkResult>("process_item", items[1].clone()).spawn().await?;
    let h3 = ctx.rpc::<WorkResult>("process_item", items[2].clone()).spawn().await?;

    // Fan-in: collect all results — each is individually durable
    let r1 = h1.await?;
    let r2 = h2.await?;
    let r3 = h3.await?;

    Ok(vec![r1, r2, r3])
}

/// Process a single work item.
/// Each invocation is individually durable — if the process crashes,
/// completed items are replayed from the log, not re-executed.
#[resonate::function]
async fn process_item(item: WorkItem) -> Result<WorkResult> {
    let output = format!("Processed: {} (item #{})", item.data, item.id);
    Ok(WorkResult {
        id: item.id,
        output,
    })
}

#[tokio::main]
async fn main() {
    let resonate = Resonate::new(ResonateConfig {
        url: Some("http://localhost:8001".into()),
        ..Default::default()
    });

    resonate.register(fan_out_fan_in).unwrap();
    resonate.register(process_item).unwrap();

    // Keep the process alive to receive work from the server.
    println!("Worker started. Waiting for invocations...");
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl-c");
}
