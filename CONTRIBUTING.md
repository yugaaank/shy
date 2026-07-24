# Contributing to shy

## Getting Started

This project uses [Rust](https://www.rust-lang.org) and [Cargo](https://doc.rust-lang.org/cargo/). To get started:

1. Clone the repository:
   ```bash
   git clone https://github.com/yugaaank/shy.git
   cd shy
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Test the code:
   ```bash
   cargo test
   ```

## Running Tests

Tests are run using `cargo test`:

```bash
cargo test
```

## Code Style

This project follows Rust's idiomatic style as outlined in the [Rust Book](https://doc.rust-lang.org/book/ch03-01-guessing-game.html#comments-vs-rustdoc).

We use `rustfmt` for code formatting:

```bash
cargo fmt --check
cargo fmt
```

And `clippy` for linting:

```bash
cargo clippy
cargo clippy --fix
```

## Troubleshooting

### Project Won't Build

If you encounter build errors:

1. Check that you're using the correct Rust version:
   ```bash
   rustup show active-toolchain
   ```

2. Update dependencies if needed:
   ```bash
   cargo update
   ```

3. Clean and rebuild:
   ```bash
   cargo clean && cargo build
   ```

### Events Not Processing

The shy daemon listens to Hyprland IPC events. Make sure:

1. Hyprland is running
2. The IPC socket is accessible
3. You have proper permissions

### Configuration Issues

The configuration file is located at `~/.config/shy/config.toml`. Common issues:

- Ensure the directory exists: `mkdir -p ~/.config/shy`
- Check file permissions and ensure it's readable

## Testing

While there aren't extensive unit tests, you can run the existing tests with:

```bash
cargo test -- --nocapture
```

## Reporting Issues

Please report bugs and feature requests through the [GitHub Issues](https://github.com/yugaaank/shy/issues) page.

When filing an issue, please include:

- Your operating system
- Hyprland version  
- shy version (if installed)
- Steps to reproduce the issue
- Log output from `debug = true` in your config

## Pull Requests

Contributions are welcome! When submitting a pull request:

1. Fork the repository
2. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. Commit your changes:
   ```bash
   git add .
   git commit -m "feat: your commit message"
   ```
4. Push to your branch:
   ```bash
   git push origin feature/your-feature-name
   ```
5. Submit a pull request

## Code of Conduct

This project follows the standard open source conduct. Please be respectful and helpful in all interactions.

## License

By contributing, you agree that your contributions will be licensed under the project's MIT License.