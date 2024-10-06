# Metric Time

A Rust crate for flexible and efficient time-based event emission with customizable intervals and event limits.

## Features

- 🕒 Precise time-based event emission
- ⚙️ Customizable time intervals
- 🔄 Event count limiting
- 🔌 Easy unsubscribe mechanism
- 🧵 Thread-safe operation
- ⚡ Async support
- 🌐 WebAssembly (WASM) compatible
- 🔄 Unified API for both native and WASM environments

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
metric_time = "0.1.0"
```

## Quick Start

```rust
use metric_time::{TimeEmitter, TimeEmitterSettings};
use std::time::Duration;

fn main() {
    // Create a TimeEmitter with custom settings
    let settings = TimeEmitterSettings::new()
        .set_max_events(5)
        .set_interval(Duration::from_secs(1));
    
    // Start the emitter with our callback
    let emitter = TimeEmitter::start(settings, |time, ctx| {
        println!(
            "Event {}: Time: {} [{}ms]",
            ctx.index,
            time.to_string(),
            ctx.settings.interval().as_millis()
        );
    });

    // Let it run for a while
    std::thread::sleep(Duration::from_secs(3));

    // Stop the emitter when done
    emitter.stop().unwrap();
}
```

## Usage

### Basic TimeEmitter

```rust
// Create a TimeEmitter with default settings
let emitter = TimeEmitter::new(|time, ctx| {
    println!("Event at: {}", time);
});
```

### Custom Settings

```rust
use std::time::Duration;

// Create custom settings
let settings = TimeEmitterSettings::new()
    .set_max_events(10)
    .set_interval(Duration::from_millis(500));

// Start the emitter with custom settings
let emitter = TimeEmitter::start(settings, |time, ctx| {
    println!("Event at: {}", time);
});
```

### With Event Limit

```rust
// Create settings with an event limit
let settings = TimeEmitterSettings::new()
    .set_max_events(5);

// Start the emitter with the event limit
let emitter = TimeEmitter::start(settings, |time, ctx| {
    println!("Event #{}: {}", ctx.index + 1, time);
});
```

### Custom Interval

```rust
// Create settings with a custom interval
let settings = TimeEmitterSettings::new()
    .set_interval(Duration::from_millis(100));

// Start the emitter with the custom interval
let emitter = TimeEmitter::start(settings, |time, ctx| {
    println!("Fast event: {}", time);
});
```

### Async Usage

```rust
#[tokio::main]
async fn main() {
    // Create settings with an event limit
    let settings = TimeEmitterSettings::new()
        .set_max_events(5);
    
    // Start the emitter with the event limit
    let emitter = TimeEmitter::start(settings, |time, _| {
        println!("Async event at: {}", time);
    });

    tokio::time::sleep(Duration::from_secs(5)).await;
    emitter.stop().unwrap();
}
```

### WebAssembly (WASM) Support

The `TimeEmitter` automatically uses the appropriate implementation based on the target platform:

```rust
// This code works the same in both native and WASM environments
let emitter = TimeEmitter::start(TimeEmitterSettings::default(), |time, _| {
    // In WASM, you might use this with web_sys to update the DOM
    println!("Current time: {}", time);
});
```

## API Reference

### TimeEmitter

- `new(callback)` - Creates a new TimeEmitter with default settings and starts it
- `start(settings, callback)` - Creates and starts a TimeEmitter with custom settings
- `settings()` - Gets the current settings
- `stop()` - Stops the time emitter

### TimeEmitterSettings

- `new()` - Creates new Settings with default values
- `set_max_events()` - Sets maximum number of events to emit
- `clear_max_events()` - Removes the maximum events limit
- `set_interval()` - Sets the time interval between events
- `set_kind()` - Sets the time kind (Base10, Base12, Base24)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Development

### Running Tests

```bash
cargo test
```

### Building Documentation

```bash
cargo doc --no-deps --open
```

## Requirements

- Rust 1.56 or higher
- Cargo

## Dependencies

- tokio (for async support in native environment)
- gloo-timers (for WASM support)
- web-sys (for WASM browser APIs)

## Known Issues

- Timing-sensitive tests might be occasionally flaky on heavily loaded systems

## Future Plans

- [ ] Add more granular control over timing
- [ ] Implement custom error handling
- [ ] Add more time format options
- [ ] Support for distributed systems
- [ ] Enhanced WASM bindings for web frameworks

## Contact

If you have any questions or feedback, please open an issue on the GitHub repository.
