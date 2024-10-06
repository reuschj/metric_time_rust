# Metric Time

A Rust crate for flexible and efficient time-based event emission with customizable intervals and event limits.

## Features

- 🕒 Precise time-based event emission
- ⚙️ Customizable time intervals
- 🔄 Event count limiting
- 🔌 Easy unsubscribe mechanism
- 🧵 Thread-safe operation
- ⚡ Async support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
metric_time = "0.1.0"
```

## Quick Start

```rust
use metric_time::TimeEmitter;
use std::time::Duration;

fn main() {
    // Create a basic emitter
    let emitter = TimeEmitter::new();

    // Set up emission with custom settings
    let unsubscribe = emitter
        .custom_settings(Settings::new()
            .set_max_events(5)
            .set_interval(Duration::from_secs(1)))
        .emit(|time, ctx| {
            println!(
                "Event {}: Time: {} [{}ms]",
                ctx.index,
                time.format("%H:%M:%S"),
                ctx.settings.interval.as_millis()
            );
        });

    // Let it run for a while
    std::thread::sleep(Duration::from_secs(3));

    // Unsubscribe when done
    let handle = unsubscribe();
    handle.join().unwrap();
}
```

## Usage

### Basic TimeEmitter

```rust
let emitter = TimeEmitter::new();
let unsubscribe = emitter.emit(|time, ctx| {
    println!("Event at: {}", time);
});
```

### Custom Settings

```rust
use std::time::Duration;

let settings = Settings::new()
    .set_max_events(10)
    .set_interval(Duration::from_millis(500));

let emitter = TimeEmitter::new().custom_settings(settings);
```

### With Event Limit

```rust
let emitter = TimeEmitter::new()
    .custom_settings(Settings::new().set_max_events(5));
```

### Custom Interval

```rust
let emitter = TimeEmitter::new()
    .custom_settings(Settings::new()
        .set_interval(Duration::from_millis(100)));
```

### Async Usage

```rust
#[tokio::main]
async fn main() {
    let emitter = TimeEmitter::new();
    let unsubscribe = emitter.emit(|time, _| {
        println!("Async event at: {}", time);
    });

    tokio::time::sleep(Duration::from_secs(5)).await;
    let handle = unsubscribe();
    handle.join().unwrap();
}
```

## API Reference

### TimeEmitter

- `new()` - Creates a new TimeEmitter with default settings
- `custom_settings()` - Configures the emitter with custom settings
- `emit()` - Starts event emission with the default settings
- `emit_with_settings()` - Starts event emission with custom settings

### Settings

- `new()` - Creates new Settings with default values
- `set_max_events()` - Sets maximum number of events to emit
- `clear_max_events()` - Removes the maximum events limit
- `set_interval()` - Sets the time interval between events

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

- chrono
- tokio (for async support)

## Known Issues

- Timing-sensitive tests might be occasionally flaky on heavily loaded systems

## Future Plans

- [ ] Add more granular control over timing
- [ ] Implement custom error handling
- [ ] Add more time format options
- [ ] Support for distributed systems

## Contact

If you have any questions or feedback, please open an issue on the GitHub repository.
