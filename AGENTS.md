# AGENTS.md

Developer and agent engineering notes for `tic`.

`tic` is a high-performance native Rust CLI over the TIC (tic.io) search and business intelligence datasets API, built to be driven by humans and autonomous LLM agents. It replaced a .NET global tool prototype (`Tic.Console`) by Niels Bosma and maintains full interface fidelity while adding SpaceCorps agentic features:
- Sub-3ms cold start speed (compiled with LTO and zero runtime dependencies)
- Native OS keystore integration (macOS Keychain, Windows DPAPI, Linux Secret Service)
- Multi-account management and safety guardrails
- YAML terminal output by default and structured JSON via `--json`
- Self-documenting agent manual directly in the binary via `tic agent-readme [--json]`

For the manual the *agent* reads, run `tic agent-readme` — that text lives in `src/readme.rs` and is the tool's interface for its primary automation audience. This file is for developers and agents maintaining or extending the codebase.

## Development & Testing Commands

```bash
cargo build --release              # target/release/tic
cargo test --locked                # unit tests + tests/cli.rs against mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install on PATH
```

### Testing Isolation

Use throwaway environment variables when testing so tests never touch host credentials:

```bash
export TIC_CONFIG_DIR=$(mktemp -d) TIC_SECRET_STORE=plaintext TIC_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
|:---|:---|
| `TIC_CONFIG_DIR` | Overrides the config directory (`config.yaml` / `.lock`) |
| `TIC_SECRET_STORE` | Forces a backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `TIC_ALLOW_PLAINTEXT_STORE=1` | Permits the plaintext fallback when no OS keystore exists |
| `TIC_API_URL` | Overrides the API base URL — used by `tests/cli.rs` for mock server |
| `TIC_API_KEY` | Overrides API token if neither `--account` nor `--api-key` is supplied |

## Codebase Layout

```
src/
  main.rs          arg parsing, --json pre-scan, clap error formatting
  cli.rs           clap hierarchy and arguments (company, person, vehicle, bankruptcy, search, login, accounts, agent-readme)
  commands/
    mod.rs         dispatch and print helper
    company.rs     company search, get, graph, parties, debtor-summary, beneficial-owners, intelligence, etc.
    person.rs      person search, get, companies
    vehicle.rs     vehicle search
    bankruptcy.rs  bankruptcy search
    search.rs      generic collection search
    login.rs       interactive/keystore login
    accounts.rs    accounts add, list, test, remove
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode mapping, error message extraction
  error.rs         ErrorCode enum and Error { code, message, detail, remediation } envelope
  output.rs        YAML by default, JSON with --json, write_error, obj! macro
  account.rs       credential resolution priority (--api-key -> --account -> TIC_API_KEY)
  config.rs        config.yaml storage, atomic writes, cross-process lock
  secrets.rs       OS keystores (Keychain, DPAPI, Secret Service, plaintext)
  readme.rs        agent-readme embedded manual and rules
tests/
  cli.rs           in-process TCP mock HTTP server and full integration suite
docs/
  index.html       dark glassmorphism landing page with Schema.org JSON-LD
  index.md         markdown homepage for LLM crawlers
  llms.txt         standard LLM tool summary
  llms-full.txt    exhaustive agent operating manual
  about.html       project background and Niels Bosma prototype credit
  contact.html     support and issue tracker links
  privacy.html     zero telemetry disclosure
  pricing.md       MIT licensing and data platform pricing notes
  auth.md          authentication and credential storage guide
  404.html         not found page
  robots.txt       search engine and crawler configuration
  sitemap.xml      sitemap for GitHub Pages
  og-image.png     social preview card
```

## Architectural Guidelines

1. **Blocking HTTP over Tokio**: A CLI typically makes 1–3 sequential calls. Avoiding Tokio removes runtime bloat and ensures 1–3 ms cold-start execution. Parallel checks (e.g. `accounts list --check`) use `std::thread::scope`.
2. **OS Keystores via Standard Tools**: macOS Keychain uses `/usr/bin/security` to prevent code-signing prompt loops on unsigned or rebuilt binaries.
3. **Structured Outputs**: Empty responses return `{"status": "ok"}` so stdout is never empty and always parses as valid YAML/JSON.
4. **Error Envelopes**: Errors always emit structured JSON/YAML to stderr and exit with non-zero codes corresponding to the `ErrorCode` enum.
