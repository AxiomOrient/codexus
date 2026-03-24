---
name: repo-cleanup-guard
description: Keep a repository simple during cleanup, file moves, codegen-input placement, and doc/tooling reorganization. Use when deciding where inputs, tools, and generated outputs belong, when reducing root clutter, or when separating human-facing docs from agent-facing instructions.
---

# Repo Cleanup Guard

Use this skill when changing repository layout or deciding where files should live.

## Goals

- Keep the root minimal and readable.
- Place files by ownership and lifecycle.
- Keep inputs, tools, and generated outputs in stable locations.
- Keep human-facing docs separate from agent-facing instructions.

## Generic Principles

- Human docs: `README.md`, `docs/`
- Repo tooling: `tools/`
- Product code: language/package root such as `crates/`, `packages/`, or `src/`
- Project-local inputs: near the owning package
- Generated outputs: near the code that compiles against them

## Rules

1. Do not add new root folders unless they are clearly workspace-global.
2. If an input belongs to one package, move it near that package instead of leaving it at the root.
3. Keep generated code near the owning code, not in `docs/`, `tools/`, or the root.
4. Keep human-facing docs in `README.md` and `docs/`.
5. Keep agent-only instructions in `AGENTS.md` or this skill, not in human docs.
6. Delete stale duplicate docs instead of keeping parallel versions.

## Working Procedure

1. Read the local cleanup guide and `AGENTS.md` if they exist.
2. Identify each changed path as one of: human doc, agent instruction, repo tool, package-local input, generated output, or temporary noise.
3. Move files to the simplest ownership-aligned layout.
4. Update every path reference in code, tests, docs, and agent guidance.
5. Run:
   - `cargo run -p xtask -- protocol-codegen`
   - `cargo fmt --all --check`
   - `cargo test --workspace`
6. Report remaining structural inconsistencies briefly.

## Decision Heuristic

When choosing between root-level and package-level placement, prefer package-level unless more than one package genuinely owns the asset.

## Prompt Template

Use this working prompt when applying the skill:

`Simplify this repository. Decide placement by ownership and lifecycle, keep the root minimal, separate human docs from agent instructions, move project-local inputs near the owning package, keep generated outputs near compiled code, update all references, then run the repository verification commands.`
