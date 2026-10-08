# Contributing

## Publish the extension

### Publish simdref to the Zed registry the first time

1. Increase `version` in `extension.toml`. Commit it.
2. Make a git tag with the same version: `git tag v0.1.0 && git push origin v0.1.0`.
3. Fork https://github.com/zed-industries/extensions on GitHub.
4. Clone the fork. In the clone, add the submodule:

   ```sh
   git submodule add https://github.com/simd-labs/zed-simdref.git extensions/simdref
   git -C extensions/simdref checkout v0.1.0
   ```

5. Add the entry to `extensions.toml`, in alphabetical sequence:

   ```toml
   [simdref]
   submodule = "extensions/simdref"
   version = "0.1.0"
   ```

   The `version` value must equal `version` in `extension.toml`.
6. Check the edit: `pnpm install`, then `pnpm sort-extensions` (it must not change the file) and `pnpm test`.
7. Commit, push to the fork, open a PR against https://github.com/zed-industries/extensions.

The full instructions: https://zed.dev/docs/extensions/publishing/publishing-guide

### Ship an update

1. Increase `version` in `extension.toml`. Commit and tag.
2. In the extensions fork, update the submodule to the new tag:

   ```sh
   git -C extensions/simdref fetch --tags
   git -C extensions/simdref checkout v0.2.0
   ```

3. Increase the `version` in the `[simdref]` block of `extensions.toml`.
4. Run `pnpm test`. Commit, push, open a PR.

The update instructions: https://zed.dev/docs/extensions/publishing/updating-and-maintenance
