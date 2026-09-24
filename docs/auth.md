---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the Tic CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for Tic CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the Tic CLI.

## Overview
The Tic CLI interfaces directly with the TIC REST API (`https://api.tic.io/`). Authentication is token-based, using API keys issued by TIC. Keys can be stored in the host operating system's native keychain or supplied directly via environment variables, CLI flags, or standard input.

## Prerequisites
- A TIC account and API key ([tic.io](https://tic.io))
- Tic CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/Tic-Cli --locked`)

## Authentication Methods

### 1. Interactive Keystore Login (`tic login`)
Recommended for local developer workstations:
```bash
tic login [account_name]
```
1. Launches the TIC portal in your browser.
2. Prompts securely for your API token in the terminal.
3. Verifies the token against the API.
4. Encrypts and stores the token in your OS keystore (macOS Keychain, Windows DPAPI, Linux Secret Service).

### 2. Stored Accounts (`tic accounts add`)
Add named accounts directly:
```bash
tic accounts add prod --api-key "$TIC_API_KEY"
```
Or via stdin (ideal for shell scripts and CI/CD):
```bash
echo "$TIC_API_KEY" | tic accounts add prod --api-key-stdin
```

### 3. Environment Variable (`TIC_API_KEY`)
Set `TIC_API_KEY` in your shell or container environment:
```bash
export TIC_API_KEY="your-api-key"
tic company get 556792-6687
```

### 4. Per-Command Flag (`--api-key`)
Explicitly pass the key with any command:
```bash
tic company get 556792-6687 --api-key "your-api-key"
```

## Multi-Account Management
```bash
tic accounts list [--check]
tic accounts test <account_name>
tic accounts remove <account_name> --yes
```

## Exit Codes & Errors
Commands exit with standardized exit codes:
- `0`: Success (`ok`)
- `1`: Unclassified error (`error`)
- `2`: Network or timeout (`network`)
- `3`: Missing or rejected key (`auth_required`)
- `4`: Resource not found (`not_found`)
- `5`: Rate limit exceeded (`rate_limited`)
- `6`: Invalid parameters (`invalid_input`)
- `7`: No account configured (`no_account`)
