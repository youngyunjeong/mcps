# MCPs

A curated collection of high-performance, reusable Model Context Protocol (MCP) servers tailored for AI coding agents.
Works seamlessly with any agent harness supporting standard MCP stdio communication: Google Antigravity (AGY), Claude Code, Claude Desktop, Cursor, Codex, and Windsurf.

## Philosophy

Most off-the-shelf MCP servers introduce heavy runtime footprints (Node.js/V8 cold starts, Python virtual environments, dozens of nested dependencies).
The servers in this collection are built with a different philosophy:

1. **Zero Runtime Dependencies**: Written in native languages like Rust with standalone binary distribution.
2. **Instant Startup (< 1ms)**: No interpreter warmup or JIT compilation; eliminates agent loop delays.
3. **Project Agnostic**: Completely free of project-specific paths, environment variables, credentials, or domain assumptions.
4. **Deterministic & Safe**: Atomic filesystem operations, strict error contracts, and zero silent failures.

## Catalog

### Filesystem

| Server | Language | Description | Link |
| :--- | :--- | :--- | :--- |
| **`fast-file-editor`** | Rust | Ultra-fast file viewer without payload truncation (bypasses 800-line/45KB limits) and atomic unique string replacement (`str_replace`) that eliminates line-drift re-read penalties. | [View Docs](servers/filesystem/fast-file-editor/) |

*(More categories and servers will be added as common agent bottlenecks are identified).*

## Quick Start

### 1. Build & Install a Server

Each server in `servers/` is self-contained. To build and install a Rust-based server:

```bash
# Clone the repository
git clone https://github.com/youngyunjeong/mcps.git
cd mcps/servers/filesystem/fast-file-editor

# Build optimized release binary (~390 KB)
cargo build --release

# Install to your PATH
cp target/release/fast-file-editor ~/.local/bin/
```

### 2. Client Setup

Configure your coding agent harness to launch the server over `stdio`:

<details>
<summary><b>Google Antigravity (AGY)</b></summary>

Add to `~/.gemini/config/mcp_config.json`:

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "fast-file-editor",
      "args": []
    }
  }
}
```
</details>

<details>
<summary><b>Claude Desktop / Claude Code</b></summary>

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "fast-file-editor",
      "args": []
    }
  }
}
```
</details>

<details>
<summary><b>Cursor</b></summary>

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "fast-file-editor"
    }
  }
}
```
</details>

## Repository Layout

```
mcps/
├── README.md               # Collection catalog, installation matrix, and agent configs
├── AGENTS.md               # Contributor and AI agent maintenance rules
├── CLAUDE.md -> AGENTS.md  # Agent guidelines symlink
└── servers/
    └── <category>/
        └── <server-name>/
            ├── README.md   # Server-specific deep dive, tool schemas, and benchmarks
            ├── Cargo.toml  # Package manifest
            └── src/        # Server implementation
```

## Contributing & Development

See [`AGENTS.md`](AGENTS.md) for guidelines on adding new servers, coding standards, and portability requirements.

## License

MIT
