# BitPet CLI

CLI tool to create and manage your BitPet (https://bitpet.dev).

## Development Environment

### Prerequisites
- Rust (latest stable version)
- Cargo (comes with Rust..)

### Quick Setup

1. **Install project dependencies:**
   ```bash
   cargo build
   ```
   This will download and compile all dependencies listed in `Cargo.toml`.


2. **Run the CLI:**
   ```bash
   cargo run -- --help   # Run with arguments
   ```

## Release new version
- Update version in `Cargo.toml`
- Push to the `main` branch
- Add a new tag that corresponds to the new version with a `v` prefix. For example, if the new version is `0.1.0`, the tag should be `v0.1.0`. This will run a circleci workflow that will build the binary and release it to the GitHub Releases page.