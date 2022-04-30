# Homebrew

## Releasing new version

Go to the Homebrew formula directory:

```shell
cd $(dirname $(brew edit --print-path mjhanninen/sour/nreplops-tool))
```

Update the formula with a helper script:

```shell
bin/release-nreplops-tool $VERSION_TRIPLET
```

Check by installing/upgrading the formula locally

```shell
brew upgrade mjhanninen/sour/nreplops-tool
```

Check version:

```shell
nr --version
```

Publish:

```shell
git push
```
