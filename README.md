# Earthquake 🌋

A high-performance credential stuffing framework for security professionals.

## Overview

Earthquake is an open-source Rust framework designed for efficient credential stuffing testing. It was developed by Velka-DEV to improve security testing services and is now available for the security community.

## Features

- **High Performance**: Multi-threaded architecture with configurable thread count for maximum efficiency
- **Combo Management**: Parse username:password combinations with configurable separators and regex filtering
- **Proxy Support**: Optional HTTP/SOCKS4/SOCKS5 proxy integration with file or URL sources
- **State Control**: Pause, resume, and stop operations with automatic progress tracking
- **Statistics**: Real-time metrics including CPM (checks per minute), hit rate, ETA, etc.
- **Retry System**: Ensures no lines are skipped due to network errors
- **Organized Results**: Saves results to categorized files by result type
- **Configuration**: Save/load configurations for quick startup

## Quickstart

### Installation

```bash
# Clone the repository
git clone https://github.com/pentech/earthquake.git
cd earthquake

# Build the project
cargo build --release
```

### Sample Usage

Here's a minimal example of using the framework:

```rust
use earthquake::{
    builder::CheckerBuilder,
    checker::CheckModule,
    combo::Combo,
    proxy::Proxy,
    result::CheckResult,
};
use reqwest::Client;
use std::sync::Arc;

struct MyModule;

#[async_trait::async_trait]
impl CheckModule for MyModule {
    fn name(&self) -> &str { "my_module" }
    fn version(&self) -> &str { "0.1.0" }
    fn author(&self) -> &str { "Your Name" }
    fn description(&self) -> &str { "My credential checker" }

    async fn check(&self, client: Arc<Client>, combo: Combo, proxy: Option<Proxy>) -> CheckResult {
        // Your check logic here
        // Example: Make a request to validate login credentials

        // Return appropriate result
        CheckResult::hit().with_message("Login successful")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure checker
    let checker = CheckerBuilder::new("my_module")
        .with_threads(100)
        .with_combo_file("combos.txt")?
        .with_proxy_file("proxies.txt")?
        .with_check_module(Arc::new(MyModule))
        .build()?;

    // Start the checker
    checker.start().await?;

    // Wait until finished or handle pause/resume/stop

    Ok(())
}
```

## Project Structure

- `src/builder.rs` - Builder pattern for checker configuration
- `src/checker.rs` - Core checker implementation
- `src/combo.rs` - Combo parsing and provider implementations
- `src/config.rs` - Configuration management
- `src/error.rs` - Error handling
- `src/proxy.rs` - Proxy management
- `src/result.rs` - Result type definitions
- `src/stats.rs` - Statistics tracking
- `src/util.rs` - Utility functions

## Security Notice

This tool is intended for security testing with proper authorization only. Unauthorized credential stuffing is illegal and unethical. Always obtain proper permission before conducting security tests.

## License

This project is licensed under the GPLv3 License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
