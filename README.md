<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/banner-dark.png">
    <source media="(prefers-color-scheme: light)" srcset="./assets/banner-light.png">
    <img alt="Fan-out / Fan-in — Resonate example" src="./assets/banner-dark.png">
  </picture>
</p>

# Fan-out / Fan-in — Resonate Rust SDK

Demonstrates fan-out/fan-in parallelism using the Resonate Rust SDK.
Multiple leaf functions are spawned in parallel via `.spawn()`, and the workflow collects their results through durable handles.

## What this demonstrates

- Spawning parallel durable executions with `ctx.run().spawn()`
- Collecting results from multiple `DurableFuture` handles
- Automatic recovery of parallel work items after crashes
- Each spawned item is individually durable — completed items are never re-executed

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.75+)
- Resonate Server & CLI: `brew install resonatehq/tap/resonate`

## Setup

```bash
git clone https://github.com/resonatehq-examples/example-fan-out-fan-in-rs.git
cd example-fan-out-fan-in-rs
```

## Run it

### 1. Start the Resonate Server

```bash
resonate dev
```

### 2. Start the worker

In a second terminal:

```bash
cargo run
```

You should see:

```
Worker started. Waiting for invocations...
```

### 3. Invoke the function

In a third terminal:

```bash
resonate invoke fanout-1 --func fan_out_fan_in --arg '[{"id":1,"data":"alpha"},{"id":2,"data":"beta"},{"id":3,"data":"gamma"}]'
```

### 4. Observe the result

Check the execution tree to see the parallel call graph:

```bash
resonate tree fanout-1
```

## What to observe

- The workflow spawns three parallel leaf functions — each is independently durable
- `resonate tree fanout-1` shows the fan-out structure: one parent promise with three child promises
- Try killing the worker after one or two items have completed and restarting — completed items are replayed from the durable log, only pending items re-execute
- The final result contains all three processed items regardless of crashes

## The code

```rust
use resonate::prelude::*;

#[resonate::function]
async fn fan_out_fan_in(ctx: &Context, items: Vec<WorkItem>) -> Result<Vec<WorkResult>> {
    // Fan-out: spawn all three items in parallel
    let h1 = ctx.run(process_item, items[0].clone()).spawn().await?;
    let h2 = ctx.run(process_item, items[1].clone()).spawn().await?;
    let h3 = ctx.run(process_item, items[2].clone()).spawn().await?;

    // Fan-in: collect all results — each is individually durable
    let r1 = h1.await?;
    let r2 = h2.await?;
    let r3 = h3.await?;

    Ok(vec![r1, r2, r3])
}

#[resonate::function]
async fn process_item(item: WorkItem) -> Result<WorkResult> {
    let output = format!("Processed: {} (item #{})", item.data, item.id);
    Ok(WorkResult { id: item.id, output })
}
```

Parallel durable execution in a handful of lines.

## File structure

```
example-fan-out-fan-in-rs/
├── Cargo.toml          # Dependencies
├── src/
│   └── main.rs         # Fan-out workflow + leaf function + worker setup
├── LICENSE             # Apache-2.0
└── README.md           # This file
```

## License

Apache-2.0
