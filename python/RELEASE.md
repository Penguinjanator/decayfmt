# Releasing the Python package

The wheel build + publish is fully automated; a release is a version bump and a tag.

## One-time setup (PyPI)

1. Create the PyPI project once (name `decayfmt-py`) on <https://pypi.org>.
2. Add a trusted publisher for the repo: PyPI → project → Publishing →
   "Add a new pending publisher", with:
   - Owner: `aravpanwar`, Repository: `decayfmt`
   - Workflow name: `python.yml`
   - Environment name: `pypi`
3. Create a GitHub environment named `pypi` (repo → Settings → Environments)
   so the publish job's `environment: pypi` gate is satisfied. No secrets are
   needed — publishing uses OpenID Connect.

## Releasing a new version

1. Bump `version` in **both** `python/Cargo.toml` and `python/pyproject.toml`
   in one commit. The two must match; `smoke.py` asserts the Cargo one.
2. Tag and push:

   ```bash
   git tag py-v0.1.0
   git push origin py-v0.1.0
   ```

3. The `python` workflow builds and smoke-tests wheels on five platforms
   (manylinux x86_64 + aarch64, macOS arm64 + x86_64, Windows msvc), then the
   `publish` job uploads them to PyPI.

   PyPI tags are deliberately separate from CLI release tags (`v*`), so
   Python versions and CLI versions can ship independently.

4. Verify:

   ```bash
   python -m venv /tmp/decayfmt-check && /tmp/decayfmt-check/bin/pip install -U decayfmt-py
   /tmp/decayfmt-check/bin/python python/tests/smoke.py
   ```

## Local build (optional)

```bash
python3 -m venv python/.venv && python/.venv/bin/pip install -U pip maturin
python/.venv/bin/maturin develop --manifest-path python/Cargo.toml
python/.venv/bin/maturin build --release --manifest-path python/Cargo.toml
```
