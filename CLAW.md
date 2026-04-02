# CLAW.md

This file provides guidance to Claw Code when working with code in this repository.

## Detected stack

- Languages: Rust (primary), Python (support/tooling).
- Frameworks: none detected from the supported starter markers.

## Verification

- Run Rust verification from root: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- Build release: `cargo build --release`
- `python/` contains Python support scripts; run tests with `python3 -m unittest discover -s python/tests -v`

## Repository shape

- `crates/` contains the Rust workspace with all CLI/runtime implementation.
- `python/` contains Python support/tooling scripts (secondary).
- `tests/` contains Python validation surfaces.
- `docs/` contains documentation and release notes.

## API Keys & Environment Variables

When running commands that call LLM providers (OpenAI, Anthropic, xAI, etc.), you must provide valid API keys.

### Setting up API keys

Use environment variables to manage API keys securely:

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"  # optional

# xAI (Grok)
export XAI_API_KEY="xai-..."
export XAI_BASE_URL="https://api.x.ai"  # optional
```

### Security best practices

- ⚠️ **Never commit API keys to the repository**
- Use `.env` files for local development (add to `.gitignore`)
- Use secret management tools in CI/CD pipelines
- Rotate keys periodically

### OAuth authentication

For Claw provider, OAuth login is available:

```bash
claw login    # Start OAuth flow
claw logout   # Clear saved credentials
```

## License

This project is released under the **MIT License**.

### Distribution requirements

When redistributing (e.g., creating Docker images, binaries, or derivative works):

1. Include a copy of the [LICENSE](LICENSE) file in your distribution
2. Preserve copyright notices and license text
3. Clearly state any modifications made

See the [LICENSE](LICENSE) file for full terms.

## Working agreement

- Prefer small, reviewable changes and keep generated bootstrap files aligned with actual repo workflows.
- Keep shared defaults in `.claw.json`; reserve `.claw/settings.local.json` for machine-local overrides.
- Do not overwrite existing `CLAW.md` content automatically; update it intentionally when repo workflows change.
- Rust is the primary implementation language; Python is for tooling and prototyping only.
