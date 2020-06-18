# Memory Logger

A logger that stores entries in memory, allowing late and sporadic consumption.

[![Cargo](https://img.shields.io/crates/v/memory_logger.svg)](https://crates.io/crates/memory_logger)
[![Documentation](https://docs.rs/memory_logger/badge.svg)](https://docs.rs/memory_logger)

## Features

- Two flavors: blocking and asynchronous.
- Optional target matching using Regex.
- Simple design, few dependencies, very fast to compile.
- No unsafe code.

### Blocking

Both logging and reading use a mutex around a single buffer, and therefore may block. This
should be good enough for most scenarios, and has a smaller memory overhead and better
locality (single buffer).

```rust
use regex::Regex;

use memory_logger::blocking::MemoryLogger;


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let target = Regex::new("^mycrate::my_module")?; // optional
	let logger = MemoryLogger::setup(log::Level::Info, target)?;

	log::info!("This is a info.");
	log::warn!("This is a warning.");

	let contents = logger.read();

	assert!(contents.contains("This is a info."));
	assert!(contents.contains("This is a warning."));
	Ok(())
}
```

### Asynchronous

Both logging and reading use a channel, and may never block. This should be faster for
high contention scenarios, but has a higher memory overhead and worse locality.

```rust
use regex::Regex;

use memory_logger::asynchronous::MemoryLogger;


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let target = Regex::new("^mycrate::my_module")?; // optional
	let logger = MemoryLogger::setup(log::Level::Info, target)?;

	log::info!("This is a info.");
	log::warn!("This is a warning.");

	let mut reader = logger.read();

	assert!(reader.next().unwrap().contains("This is a info."));
	assert!(reader.next().unwrap().contains("This is a warning."));
	Ok(())
}
```

## Simplicity

Memory logger aims to be a simple logging mechanism. There are no plans to implement
advanced features.

## Licence

`memory_logger` is licenced under the [MIT Licence](http://opensource.org/licenses/MIT).
