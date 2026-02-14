# rootfs-tests

User experience tests for LevitateOS rootfs. Uses systemd-nspawn to verify the OS works as a daily driver competing with Arch Linux.

## Status

| Metric | Value |
|--------|-------|
| Stage | Alpha |
| Target | x86_64 Linux (systemd-nspawn) |
| Last verified | 2026-01-23 |

### Works

- Container-based test execution
- Category filtering (binaries, auth, filesystem, systemd)
- Test trait for easy test addition

### Known Issues

- See parent repo issues

---

## Author

<!-- HUMAN WRITTEN - DO NOT MODIFY -->

[Waiting for human input]

<!-- END HUMAN WRITTEN -->

---

Each test answers: "Can a user do X with this OS?"

## Usage

```bash
# Run all tests
cargo run -- run

# Run with specific tarball
cargo run -- run --tarball .artifacts/out/leviso/levitateos-base.tar.xz

# Run specific category only
cargo run -- run --category auth

# Verbose output
cargo run -- run --verbose

# List all tests
cargo run -- list
```

## Test Categories

| Category | Tests |
|----------|-------|
| `binaries` | bash, coreutils, grep, sed, tar, mount, curl, recipe |
| `auth` | sudo, su, visudo, passwd/shadow, PAM |
| `filesystem` | FHS directories, symlinks, /etc configs, os-release |
| `systemd` | systemd, systemctl, journalctl, units, getty |

## Example Output

```
LevitateOS Rootfs Tests
=======================

Testing: Can a user use this as a daily driver OS?

Extracting .artifacts/out/leviso/levitateos-base.tar.xz ...
Ready.

━━━ BINARIES ━━━
  ✓ bash (0.1s)
  ✓ coreutils (0.2s)
  ✓ grep (0.1s)

━━━ AUTH ━━━
  ✓ sudo (0.3s)
  ✓ pam-config (0.1s)

════════════════════════════════════════════════════════════

✓ All 15 tests passed (2.3s)

This rootfs is ready for daily driver use.
```

## Adding Tests

1. Create a test struct in `src/tests/`
2. Implement the `Test` trait
3. Add to the category's `*_tests()` function

```rust
struct MyTest;

impl Test for MyTest {
    fn name(&self) -> &str { "my-test" }
    fn category(&self) -> &str { "binaries" }
    fn ensures(&self) -> &str { "User can run my-command" }

    fn run(&self, container: &Container) -> TestResult {
        run_test(self, || {
            container.exec_ok("my-command --version")
        })
    }
}
```

## Architecture

```
rootfs-tests/
├── src/
│   ├── main.rs           # CLI and test runner
│   ├── container.rs      # systemd-nspawn wrapper
│   └── tests/
│       ├── mod.rs        # Test trait and collection
│       ├── auth.rs       # sudo, su, PAM tests
│       ├── binaries.rs   # Binary existence tests
│       ├── filesystem.rs # FHS and config tests
│       └── systemd.rs    # Systemd service tests
```

## Requirements

- systemd-nspawn (usually in `systemd` package)
- sudo access (for container operations)
- LevitateOS tarball or extracted rootfs

## License

MIT
