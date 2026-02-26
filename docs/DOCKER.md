# Docker Usage Guide

Unicleaner provides a minimal Docker image built with a statically-linked musl binary for maximum portability and security.

## Quick Start

### Pull from GitHub Container Registry

```bash
docker pull ghcr.io/poelzi/unicleaner:latest
```

### Run with Current Directory

```bash
# Scan current directory
docker run --rm -v "$(pwd):/workspace" ghcr.io/poelzi/unicleaner:latest .

# With specific options
docker run --rm -v "$(pwd):/workspace" ghcr.io/poelzi/unicleaner:latest \
  scan . --format json > results.json
```

### Run with Custom Config

```bash
# Mount config file
docker run --rm \
  -v "$(pwd):/workspace" \
  -v "$(pwd)/unicleaner.toml:/config/unicleaner.toml:ro" \
  ghcr.io/poelzi/unicleaner:latest \
  scan . --config /config/unicleaner.toml
```

## Building Locally

### Using Nix (Recommended)

```bash
# Build static binary
nix build .#unicleaner-static

# Verify it's truly static
./scripts/verify-static.sh result/bin/unicleaner

# Build Docker image
nix build .#docker

# Load into Docker
docker load < result

# Tag and use
docker tag unicleaner:1.0.0-alpha1 unicleaner:latest
docker run --rm -v "$(pwd):/workspace" unicleaner:latest --version
```

### Using Dockerfile (Alternative)

```dockerfile
# Example Dockerfile using the static binary
FROM scratch
COPY result/bin/unicleaner /bin/unicleaner
WORKDIR /workspace
ENTRYPOINT ["/bin/unicleaner"]
CMD ["--help"]
```

Build:
```bash
nix build .#unicleaner-static
docker build -t unicleaner:custom .
```

## Image Details

### Size
The Docker image is extremely minimal:
- **Base**: `scratch` (empty base image)
- **Binary**: ~5-10 MB (statically linked musl binary)
- **Total**: ~5-10 MB uncompressed

### Security Features
- ✅ **No OS packages** - eliminates supply chain vulnerabilities
- ✅ **No shell** - prevents shell injection attacks
- ✅ **Statically linked** - no dynamic library dependencies
- ✅ **Minimal attack surface** - only contains the binary

### Verification

Check that the binary is truly static:

```bash
# Extract binary from image
docker create --name temp unicleaner:latest
docker cp temp:/bin/unicleaner ./unicleaner-extracted
docker rm temp

# Verify no dynamic dependencies
ldd ./unicleaner-extracted
# Should output: "not a dynamic executable"

# Check binary info
file ./unicleaner-extracted
# Should show: "statically linked"
```

## GitHub Actions Usage

### In Your Workflow

```yaml
name: Scan Code
on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Scan for malicious Unicode
        run: |
          docker run --rm -v "$PWD:/workspace" \
            ghcr.io/poelzi/unicleaner:latest \
            scan . --format json > scan-results.json

      - name: Check results
        run: |
          if [ -s scan-results.json ]; then
            VIOLATIONS=$(jq '.violations | length' scan-results.json)
            if [ "$VIOLATIONS" -gt 0 ]; then
              echo "Found $VIOLATIONS violations!"
              exit 1
            fi
          fi
```

### As a Composite Action

Create `.github/actions/unicleaner-scan/action.yml`:

```yaml
name: 'Unicleaner Scan'
description: 'Scan repository for malicious Unicode characters'
inputs:
  path:
    description: 'Path to scan'
    required: false
    default: '.'
  config:
    description: 'Path to config file'
    required: false
  fail-on-violations:
    description: 'Fail if violations found'
    required: false
    default: 'true'

runs:
  using: 'composite'
  steps:
    - name: Pull Unicleaner Docker image
      shell: bash
      run: docker pull ghcr.io/poelzi/unicleaner:latest

    - name: Run scan
      shell: bash
      run: |
        CONFIG_ARG=""
        if [ -n "${{ inputs.config }}" ]; then
          CONFIG_ARG="--config ${{ inputs.config }}"
        fi

        docker run --rm -v "$PWD:/workspace" \
          ghcr.io/poelzi/unicleaner:latest \
          scan ${{ inputs.path }} $CONFIG_ARG \
          --format json > scan-results.json

    - name: Check violations
      shell: bash
      if: inputs.fail-on-violations == 'true'
      run: |
        VIOLATIONS=$(jq '.violations | length' scan-results.json)
        if [ "$VIOLATIONS" -gt 0 ]; then
          echo "::error::Found $VIOLATIONS Unicode violations"
          exit 1
        fi
```

Use it in your workflow:

```yaml
jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/unicleaner-scan
        with:
          path: src/
          fail-on-violations: true
```

## CI/CD Integration Examples

### GitLab CI

```yaml
unicode-scan:
  stage: security
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker pull ghcr.io/poelzi/unicleaner:latest
    - docker run --rm -v "$PWD:/workspace"
        ghcr.io/poelzi/unicleaner:latest
        scan . --format json > results.json
    - |
      VIOLATIONS=$(jq '.violations | length' results.json)
      if [ "$VIOLATIONS" -gt 0 ]; then
        echo "Found $VIOLATIONS violations"
        exit 1
      fi
  artifacts:
    reports:
      junit: results.json
    when: always
```

### CircleCI

```yaml
version: 2.1

jobs:
  unicode-scan:
    docker:
      - image: cimg/base:current
    steps:
      - checkout
      - setup_remote_docker
      - run:
          name: Pull Unicleaner
          command: docker pull ghcr.io/poelzi/unicleaner:latest
      - run:
          name: Scan for malicious Unicode
          command: |
            docker run --rm -v "$PWD:/workspace" \
              ghcr.io/poelzi/unicleaner:latest \
              scan . --format json > results.json
      - store_artifacts:
          path: results.json

workflows:
  version: 2
  security:
    jobs:
      - unicode-scan
```

### Jenkins

```groovy
pipeline {
    agent any

    stages {
        stage('Unicode Security Scan') {
            steps {
                script {
                    docker.image('ghcr.io/poelzi/unicleaner:latest').inside('-v ${WORKSPACE}:/workspace') {
                        sh 'unicleaner scan . --format json > /workspace/results.json'
                    }

                    def results = readJSON file: 'results.json'
                    if (results.violations.size() > 0) {
                        error("Found ${results.violations.size()} Unicode violations")
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'results.json', allowEmptyArchive: true
        }
    }
}
```

## Advanced Usage

### Multi-stage Scan

```bash
# Stage 1: Quick scan of critical files
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/poelzi/unicleaner:latest \
  scan src/ --severity error

# Stage 2: Full scan with warnings
docker run --rm -v "$(pwd):/workspace" \
  ghcr.io/poelzi/unicleaner:latest \
  scan . --severity warning --format json
```

### Scan Git Changes Only

```bash
# In CI, scan only changed files
docker run --rm -v "$(pwd):/workspace" \
  -v "$(pwd)/.git:/workspace/.git:ro" \
  ghcr.io/poelzi/unicleaner:latest \
  scan . --diff
```

### Custom Entrypoint

```bash
# Run with shell for debugging (requires adding shell to image)
docker run --rm -it --entrypoint /bin/sh \
  -v "$(pwd):/workspace" \
  unicleaner:latest

# Or use busybox base for debugging
FROM busybox
COPY --from=builder /bin/unicleaner /bin/unicleaner
WORKDIR /workspace
ENTRYPOINT ["/bin/unicleaner"]
```

## Troubleshooting

### Permission Issues

```bash
# Run as current user
docker run --rm --user $(id -u):$(id -g) \
  -v "$(pwd):/workspace" \
  ghcr.io/poelzi/unicleaner:latest .
```

### Mount Issues

```bash
# Use absolute paths
docker run --rm -v "/absolute/path/to/code:/workspace" \
  ghcr.io/poelzi/unicleaner:latest .

# Check mount worked
docker run --rm -v "$(pwd):/workspace" \
  --entrypoint ls \
  ghcr.io/poelzi/unicleaner:latest \
  -la /workspace
```

### Binary Verification Failed

If you get "not found" or "no such file" errors:

```bash
# Check architecture
docker run --rm ghcr.io/poelzi/unicleaner:latest --version

# Verify static linking
docker run --rm --entrypoint sh ghcr.io/poelzi/unicleaner:latest \
  -c "ldd /bin/unicleaner || echo 'Static binary confirmed'"
```

## References

- [Nix Docker Tools Documentation](https://nixos.org/manual/nixpkgs/stable/#sec-pkgs-dockerTools)
- [Musl libc](https://musl.libc.org/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
