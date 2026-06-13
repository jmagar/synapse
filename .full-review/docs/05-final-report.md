# Documentation Audit Final Report

Date: 2026-06-13

## Gaps Found

- Root `CLAUDE.md` and `docs/CLAUDE.md` still described the repository as
  `rmcp-template`, including `Example*`, `EXAMPLE_*`, and `/v1/example`
  guidance that no longer applies to Synapse2.
- `docs/QUICKSTART.md` used the old `SYNAPSE2_*` env prefix, port `3100`, and
  obsolete CLI/MCP examples such as `flux docker-info`.
- `server.json` was an invalid JSON file because it contained an HTML comment
  preamble, and it still described `tv.tootie/example-mcp`.
- `config.example.toml` mixed Synapse2 settings with template adaptation
  instructions and stale port/name examples.
- `docs/API.md` had stale CLI examples for container exec and invalid host
  doctor check names.
- Several service docs had stale `owner: rmcp-template` /
  `scope: template` frontmatter.
- `docs/MCP-REGISTRY-PUBLISH-GUIDE.md` still described adapting a template
  manifest instead of publishing the current Synapse2 manifest.

## Updates Made

- Rewrote root and docs-local agent instructions for Synapse2 and documented
  the `docs/PATTERNS.md` family-reference exception.
- Rewrote the quickstart around current `SYNAPSE_MCP_*` variables, port `40080`,
  current CLI commands, REST endpoint, and mcporter smoke tests.
- Replaced `server.json` with valid Synapse2 registry metadata.
- Updated config, environment, API, README, xtask, CI, testing, scripts, and
  pre-commit docs where they carried stale template ownership or examples.
- Updated the MCP registry guide to use Synapse2 image, namespace, and release
  examples.

## Validation

- `jq . server.json`
- stale-reference sweep across Synapse2-specific docs and manifests
- doc symlink policy confirmed before edits: `AGENTS.md` and `GEMINI.md` point
  to their sibling `CLAUDE.md` files

Remaining note:

- `docs/PATTERNS.md` intentionally keeps generic `example` / `EXAMPLE_*`
  snippets because it is the family-level pattern catalog rather than a
  Synapse2 service guide.
