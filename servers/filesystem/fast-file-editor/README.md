# fast-file-editor

A high-performance Model Context Protocol (MCP) server written in Rust that provides fast file inspection and atomic unique string replacement.
Specifically engineered for AI coding agents to eliminate the **Line Drift Trap** and bypass payload truncation limits on large files.

---

## The Problem

### 1. The Line Drift Trap
Most default agent harnesses (such as Antigravity/AGY) rely on line-indexed editing tools (e.g. `replace_file_content(StartLine, EndLine, TargetContent)`).
While safe for single edits, this causes a severe architectural breakdown when an agent performs multiple edits across a file:

```text
Step 1: Agent reads file (1,000 lines)
Step 2: Agent edits line 150 (inserts 20 new lines)
        --> Lines below 150 now shift downward by +20 lines!
Step 3: Agent attempts to edit line 300 using original coordinates [300, 315]
        --> Fails! TargetContent is actually at line 320 now.
Step 4: To avoid failures, the agent defensively re-reads the entire file before each edit.
```

In a distributed 5-edit task, this causes **5 redundant re-reads**, injecting up to **50,000 duplicate tokens** into the context window and adding 15-20 seconds of unnecessary LLM round-trips.

### 2. Payload Truncation Caps
Default agent inspection tools often enforce strict per-call limits (e.g. at most 800 lines or 45KB).
Files larger than 45KB (common in modern frontend components, generated types, or backend services) are truncated mid-stream, forcing the agent to issue multiple range-offset queries just to understand the file structure.

---

## The Solution: `fast-file-editor`

`fast-file-editor` exposes two focused MCP tools:

1. **`str_replace`**: Claude Code-style unique substring replacement.
   - Requires **no line numbers** (`path`, `old_str`, `new_str`).
   - Line drift from prior edits has **zero impact** on subsequent edits.
   - Enforces strict uniqueness (fails safely if `old_str` is not unique).
2. **`view_file_fast`**: Instant, line-numbered file viewing without 800-line or 45KB payload truncation limits.

---

## Tool Reference

### 1. `str_replace`

Replaces an exact, uniquely occurring substring in a file.

#### Parameters
| Parameter | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `path` | `string` | Yes | Absolute path to the target file. |
| `old_str` | `string` | Yes | Exact substring to find and replace. Must occur exactly once in the file. |
| `new_str` | `string` | Yes | Replacement text to insert in place of `old_str`. |

#### Error Handling & Uniqueness Contract
* **0 Matches**: Returns an error indicating `old_str` was not found. Prompts the agent to verify indentation, whitespace, and exact characters.
* **>1 Matches**: Returns an error reporting the exact number of matches found (e.g. `appears 3 times`). Instructs the agent to provide more surrounding context lines to make the anchor unique.
* **1 Match**: Performs atomic replacement and returns:
  `Successfully replaced 1 occurrence at line <N> in '<path>'.`

#### Safety: Atomic Writes
To prevent file corruption from crashes or concurrent writes, `str_replace` writes content to a temporary file (`<path>.tmp.<pid>`) in the same directory and performs an atomic filesystem rename.

---

### 2. `view_file_fast`

Streams file contents with 1-indexed line numbers (`    1: content`).

#### Parameters
| Parameter | Type | Required | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `path` | `string` | Yes | - | Absolute path to the file. |
| `start_line` | `integer` | No | `1` | 1-indexed starting line number. |
| `limit` | `integer` | No | `2000` | Maximum number of lines to return. |

#### Output Format
```text
File: /path/to/component.tsx (total lines: 1229)
Showing lines 1 to 1229:

    1: import React from "react";
    2: import { useState } from "react";
    ...
 1229: export default MyComponent;
```

---

## Benchmark Results

All benchmarks were measured on Apple Silicon (ARM64) running release builds (`cargo build --release`, 393 KB binary).

### Benchmark 1: Large File Inspection (~1,200 lines, 58.8 KB)

| Metric | Traditional Built-in (`view_file`) | `fast-file-editor` (`view_file_fast`) | Improvement |
| :--- | :--- | :--- | :--- |
| **Tool Execution Latency** | ~5 - 15 ms | **0.80 ms (0.0008s)** | **10x+ faster** |
| **1-Call Completeness** | ❌ Truncated (capped at 800 lines / 45KB) | 🟢 **100% complete (1,229 lines)** | No truncation |
| **Tool Calls Required** | 2 calls (1-800, 801-1229) | **1 call** | 50% fewer turns |
| **Missing Content** | 429 lines truncated | **0 lines** | Complete context |

### Benchmark 2: Distributed Multi-site Editing (5 edits across 1,000 lines)

Simulated editing 5 distinct locations across a 1,000-line file where each edit inserts 5 new lines (+5, +10, +15, +20 line shifts):

| Metric | Traditional Line-Indexed Workflow | `fast-file-editor` (`str_replace`) | Improvement |
| :--- | :--- | :--- | :--- |
| **Total Tool Latency** | ~15 - 20s (including LLM re-read turns) | **2.36 ms (all 5 edits)** | Instantaneous |
| **Per-Edit Latency** | - | **0.30 - 0.52 ms** | Microsecond level |
| **Intermediate Re-reads** | **5 defensive re-reads** | **0 re-reads** | **100% eliminated** |
| **Total Tool Calls** | **10 calls** (5 reads + 5 edits) | **5 calls** (5 consecutive edits) | **50% reduction** |
| **Wasted Context Tokens** | **~50,000 tokens** (800 lines x 5) | **0 tokens** | **Zero context waste** |
| **Line-drift Error Rate** | High (fails if offset not recalculated) | **0% (uniqueness guaranteed)** | 100% deterministic |

---

## Architecture & Implementation Details

* **Single Standalone Binary**: Compiles down to **~390 KB** with zero external shared libraries.
* **Instant Startup (< 1ms)**: Starts and responds to JSON-RPC requests immediately without VM warm-up (Node.js/V8 cold starts take 50-100ms; Python takes 40-80ms).
* **Minimal Memory Footprint**: Uses **1-2 MB RAM** during execution (Node.js processes typically consume 30-50 MB).
* **Synchronous Stdio Loop**: Operates over standard input and standard output without heavy asynchronous runtime overhead (`tokio` not needed for sequential JSON-RPC turns).
* **SIMD Byte Search**: Leverages optimized byte matching for newline counting and substring location.

---

## Building & Testing

### Build Release Binary
```bash
cargo build --release
```
The optimized executable will be located at `target/release/fast-file-editor`.

### Run Automated Unit Tests
```bash
cargo test
```
The test suite validates:
- Single unique match atomic replacement.
- Error handling when `old_str` does not exist.
- Error handling when `old_str` matches multiple locations.
- Line slicing and 1-indexed formatting in `view_file_fast`.

---

## License

MIT
