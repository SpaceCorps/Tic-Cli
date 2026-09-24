# Tic CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Tic-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Tic-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Tic-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Tic-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Tic-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A blazing fast, native command-line tool and agent interface for the [TIC](https://tic.io) search and business intelligence datasets API (company, person, vehicle, and bankruptcy lookup). Built in Rust for developers and autonomous AI workflows.

---

## Highlights

- ⚡ **Sub-3ms Startup**: Compiled as a native static binary with zero runtime dependencies. Cold starts in ~1–3 ms (replacing .NET global tool startup latency).
- 🔐 **OS Keystore Integration**: `tic login` prompts for your token securely and saves it to native OS vaults (macOS Keychain, Linux Secret Service / libsecret, Windows DPAPI).
- 🏢 **Comprehensive Intelligence Surface**: Complete commands across Swedish companies (search, get, beneficial owners, ownership graph, tree hierarchy, board parties, Kronofogden debtor summary, registered vehicles, property holdings), vehicle lookups, bankruptcy records, and person lookups.
- 🤖 **AI Agent Native**: Machine-readable `--json` output, standardized error envelopes with stable numeric exit codes, and self-documenting agent guidance (`tic agent-readme`).
- 🛡️ **Flexible Authentication**: Full multi-account support (`-a, --account <name>`), direct flag override (`--api-key <key>`), or environment variable (`TIC_API_KEY`).

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Tic-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Tic-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`tic-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Tic-Cli/releases/download/v1.0.0/tic-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`tic-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Tic-Cli/releases/download/v1.0.0/tic-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`tic-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Tic-Cli/releases/download/v1.0.0/tic-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`tic-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/Tic-Cli/releases/download/v1.0.0/tic-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Quickstart

### 1. Authenticate

Authenticate interactively and store your key in the secure OS keystore:

```bash
# Interactive login (stores key under account 'work')
tic login work

# Or pass via environment variable
export TIC_API_KEY="your-api-key"

# Or configure a stored account directly
tic accounts add prod --api-key "$TIC_API_KEY"
```

### 2. Query Swedish Companies

```bash
# Look up company by registration number (with or without hyphens)
tic company get 556792-6687

# Look up company by internal TIC ID
tic company get-by-id 3325421

# Search companies by name or specific fields
tic company search "Bosma Interactive"
tic company search "+46767742725" --query-by "phoneNumbers.e164PhoneNumber"
tic company search "Fabriksgatan" --query-by "mostRecentRegisteredAddress.streetAddress"
tic company search "*" --query-by "registrationNumber" --filter-by "hasIntelligence:true" --per-page 5

# Deep intelligence & compliance datasets
tic company beneficial-owners 3325421
tic company graph 3325421
tic company tree 3325421
tic company parties 3325421
tic company debtor-summary 3325421
tic company vehicles 3325421
tic company properties 3325421
```

### 3. Query Persons, Vehicles & Bankruptcies

```bash
# Person lookup by personal identity number (use % for wildcard)
tic person search 198207174171
tic person get 1625054
tic person companies 1625054

# Vehicle search by licence plate or VIN
tic vehicle search "ABC123"
tic vehicle search "YV1XZ" --query-by vin

# Bankruptcy records search
tic bankruptcy search "5566778899"
tic bankruptcy search "*" --filter-by "initiatedDate:>=1711929600" --sort-by "initiatedDate:desc"
```

---

## Command Reference

### Global Options

| Option | Description |
|:---|:---|
| `--json` | Print raw, structured JSON instead of YAML |
| `--format <format>` | Output format (`yaml`, `json`) |
| `-a, --account <name>` | Select configured account from keystore |
| `--api-key <key>` | Specify API key directly for command |
| `--verbose` | Print HTTP request method, URL, and status code |

### Commands

| Command | Description |
|:---|:---|
| `tic company get <REG_NR>` | Look up company by registration number |
| `tic company get-by-id <COMPANY_ID>` | Look up company by internal TIC ID |
| `tic company search <QUERY>` | Search companies across name, address, phone, email |
| `tic company graph <COMPANY_ID>` | Get ownership graph network |
| `tic company tree <COMPANY_ID>` | Get corporate tree hierarchy |
| `tic company intelligence <COMPANY_ID>` | Get intelligence anomalies and warnings |
| `tic company beneficial-owners <COMPANY_ID>` | Get beneficial owners (UBO) |
| `tic company parties <COMPANY_ID>` | Get board members, CEO, auditors, signatories |
| `tic company debtor-summary <COMPANY_ID>` | Get Kronofogden unpaid debt records |
| `tic company vehicles <COMPANY_ID>` | Get vehicles registered to company |
| `tic company properties <COMPANY_ID>` | Get properties owned by company |
| `tic person search <QUERY>` | Search persons by personal identity number |
| `tic person get <PERSON_ID>` | Get person details |
| `tic person companies <PERSON_ID>` | Get all company roles for a person |
| `tic vehicle search <QUERY>` | Search Swedish vehicle registry |
| `tic bankruptcy search <QUERY>` | Search Swedish bankruptcy records |
| `tic search <COLLECTION> <QUERY>` | Search any TIC collection directly |
| `tic login [name]` | Authenticate and save token to OS keystore |
| `tic accounts add <name>` | Add named account to keystore |
| `tic accounts list [--check]` | List configured accounts |
| `tic accounts test <name>` | Test credentials for account |
| `tic accounts remove <name>` | Remove account and wipe stored secret |
| `tic agent-readme [--json]` | Self-documenting manual for AI agents |

---

## Output Formats & Exit Codes

### Output Formats
- **YAML (default)**: Clean, highly readable output for interactive terminal inspection.
- **JSON (`--json`)**: Valid structured JSON for automated pipelines, `jq`, and LLM tool execution loops.

### Exit Codes

| Exit Code | Machine Code | Meaning |
|:---|:---|:---|
| 0 | `ok` | Success |
| 1 | `error` | Unclassified error |
| 2 | `network` | Network or timeout error (retry once) |
| 3 | `auth_required` | Missing or rejected API key |
| 4 | `not_found` | Resource does not exist |
| 5 | `rate_limited` | Rate limit (429) hit; back off |
| 6 | `invalid_input` | Parameter or validation error |
| 7 | `no_account` | No account specified and none found |

---

## License

MIT License. Copyright (c) 2026 SpaceCorps.
Original .NET prototype by Niels Bosma.
