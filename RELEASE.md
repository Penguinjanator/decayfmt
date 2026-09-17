# Releasing decayfmt

Binaries, the GitHub Release, the crates.io publish, and the PyPI wheels are all
automated. A release is a version bump and one tag.

## What a `v*` tag does

Pushing a `v*` tag starts two workflows in parallel:

- `release.yml` builds and tests on five targets (Linux x86_64 + aarch64, macOS
  x86_64 + arm64, Windows msvc), attaches the archives to the GitHub Release, and
  publishes the crate to crates.io.
- `python.yml` builds and smoke-tests wheels on four platforms, then publishes
  them to PyPI.

Each publish waits for its own build matrix to pass, because neither a crates.io
version nor a PyPI version can be reused once uploaded. The two publishes are
independent, so one can succeed while the other fails; recover by fixing the
cause and releasing a new patch version rather than retrying the same one.

A `py-v*` tag publishes the Python package on its own, without cutting a new CLI
release. See [python/RELEASE.md](python/RELEASE.md).

## Releasing a new version

1. Bump `version` in `Cargo.toml`, `python/Cargo.toml`, and
   `python/pyproject.toml`. The two Python versions must match; `smoke.py`
   asserts the Cargo one.
2. Tag and push:

   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. Verify:

   ```bash
   cargo install decayfmt --force && decayfmt --version
   pip install -U decayfmt-py && python python/tests/smoke.py
   ```

## Publishing setup

Both registries use trusted publishing: a GitHub OIDC token is exchanged for a
short-lived registry token, so no API tokens are stored in the repository. This is
configured already and is recorded here so it can be rebuilt if it is ever lost.

**crates.io** (crate → Settings → Trusted Publishing):

| Field | Value |
| --- | --- |
| Repository owner | `aravpanwar` |
| Repository name | `decayfmt` |
| Workflow filename | `release.yml` |
| Environment name | empty |

**PyPI** (account → Publishing → pending publisher, until the first upload):

| Field | Value |
| --- | --- |
| PyPI project name | `decayfmt-py` |
| Owner | `aravpanwar` |
| Repository name | `decayfmt` |
| Workflow name | `python.yml` |
| Environment name | `pypi` |

The PyPI job also needs a GitHub environment named `pypi` (repo → Settings →
Environments) with no secrets and no protection rules. A required reviewer or
wait timer there would stall the publish.

A crate must already exist on crates.io before a trusted publisher can be
attached to it, so the very first version of a new crate is published manually.
