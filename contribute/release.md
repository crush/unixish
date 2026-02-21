# release

## local package

```bash
powershell -ExecutionPolicy Bypass -File scripts/package.ps1
```

output:

- `dist/unixish-windows-x64.zip`
- `dist/SHA256SUMS.txt`

## github release

1. set version in `Cargo.toml`
2. create tag: `git tag v0.0.2`
3. push: `git push origin v0.0.2`
4. workflow publishes release assets
