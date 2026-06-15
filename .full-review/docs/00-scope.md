# Documentation Audit Scope

Date: 2026-06-13

Scope:

- Root onboarding docs: `README.md`, `CLAUDE.md`, `AGENTS.md`, `GEMINI.md`
- Stable docs under `docs/*.md`
- Plugin skill docs under `plugins/synapse2/skills/synapse2/`
- Runtime examples in `config.example.toml`, `server.json`, and `xtask/README.md`

Method:

- Verified live CLI shape with `cargo run --locked -- --help`, `flux --help`,
  and `scout --help`.
- Searched for stale template markers including `EXAMPLE_*`, `SYNAPSE2_*`,
  `example-mcp`, `/v1/example`, and old `docker-info` command forms.
- Checked doc symlink policy for `AGENTS.md` and `GEMINI.md`.
- Validated `server.json` parses as JSON.

Intentional exclusions:

- Historical session logs under `docs/sessions/`.
- Family-level illustrative examples in `docs/PATTERNS.md`; `docs/CLAUDE.md`
  now documents this as the explicit exception.
