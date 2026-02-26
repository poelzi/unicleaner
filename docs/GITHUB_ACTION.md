# GitHub Action Usage

Unicleaner is published as a GitHub Action so repositories can enforce Unicode security checks in pull requests.

## Quick Start

```yaml
name: Unicode Security Check

on:
  pull_request:
    branches: [main]

permissions:
  contents: read

jobs:
  unicode-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: poelzi/unicleaner@v1
        with:
          mode: diff
          base-ref: main
          fail-on-violations: true
```

## Reusable Workflow

```yaml
jobs:
  unicode-security:
    uses: poelzi/unicleaner/.github/workflows/unicode-check.yml@v1
    with:
      mode: diff
      base-ref: main
      fail-on-violations: true
```

## Action Inputs

- `version` (default: `latest`): release tag to download, for example `v1.2.3`
- `mode` (default: `diff`): `diff` or `full`
- `path` (default: `.`): scan root when `mode=full`
- `base-ref` (default: `main`): branch to diff against when `mode=diff`
- `config` (optional): path to `unicleaner.toml`
- `severity` (optional): minimum severity filter
- `fail-on-violations` (default: `true`): fail the job if violations are found
- `output-file` (default: `unicleaner-results.json`): JSON report path

## Action Outputs

- `violation_count`
- `files_scanned`
- `output_file`

## Notes

- The action currently supports `ubuntu-latest` runners (Linux x64).
- `mode=diff` requires a git checkout with history (`fetch-depth: 0`).
- For strict enforcement, configure branch protection to require this check.

## Maintainer Checklist (GitHub.com)

To make `uses: poelzi/unicleaner@v1` work reliably for everyone:

1. Create a release tag (for example `v1.0.0`) so release assets exist.
2. Move/create a major tag alias `v1` that points to the latest v1 release commit.
3. Verify the release includes `unicleaner-<version>-linux-x86_64-musl.tar.gz`.
4. Set branch protection in this repo so the release and CI workflows must pass.
