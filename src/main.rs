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
/// Spawns multiple leaf functions in parallel using `.spawn()`,
/// then collects all results through durable handles.
/// If the process crashes mid-execution, only incomplete items re-execute.
#[resonate::function]
async fn fan_out_fan_in(ctx: &Context, items: Vec<WorkItem>) -> Result<Vec<WorkResult>> {
    // Fan-out: spawn all items in parallel
    let mut handles = Vec::new();
    for item in items {
        let handle = ctx.run(process_item, item).spawn().await?;
        handles.push(handle);
    }

    // Fan-in: collect all results
    let mut results = Vec::new();
    for mut handle in handles {
        let result = handle.await?;
        results.push(result);
    }

    Ok(results)
}

/// Process a single work item.
/// Each invocation is individually durable — if the process crashes,
/// completed items are replayed from the log, not re-executed.
#[resonate::function]
async fn process_item(item: WorkItem) -> Result<WorkResult> {
    // Simulate some processing
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
