# HachimiRedux Installer for Steam
Simple installer for [HachimiRedux](https://github.com/jalbarrang/hachimi-redux), adapted for use with the Steam version of The Honse Game (JP). Built against the latest DLLs from the [HachimiRedux](https://github.com/jalbarrang/hachimi-redux) repository.

This is a fork of the upstream Hachimi installer, extended to optionally bundle the HachimiRedux Training Tracker plugin.

# Usage
The installer supports both GUI and CLI/Unattended mode. To start in GUI mode, just launch the application without any arguments.

## CLI
- Usage: `hachimi_installer.exe [OPTIONS] <SUBCOMMAND>`
- Subcommands:
    - install
    - uninstall
- Options:
    - `--target <filename or path>`: Specifies the install target, relative to the install dir. If it's an absolute path, the install dir will be ignored.
    - `--explicit-target <filename>`: Explicitly specifies the specific target name, regardless of the target's path. This option influences the install method that will be used.
    - `--install-dir <path>`: Specifies the install directory.
    - `--sleep <milliseconds>`: Duration to sleep before starting the install process.
    - `--prompt-for-game-exit`: When enabled, the installer will display a dialog prompting the user to close the game if it is running. The dialog will continue to display until the user closes the game, or cancel the install process.
    - `--pre-install`: Also run pre-install checks. Ignored when uninstalling.
    - `--post-install`: Also run post-install tasks. Ignored when uninstalling.
    - `--with-training-tracker`: Accepted but a **no-op** (kept so older host updaters that pass it still work). Training Tracker now ships inside `hachimi.dll`; on install the installer also removes any stale standalone `hachimi_training_tracker.dll` + its `load_libraries` entry left by an older installer.
    - `--launch-game`: Launch the game after the operation finishes successfully.
    - `--`: Arguments separator; any arguments put after it will be passed onto the game when using `--launch-game`.

# Building
Put `hachimi.dll` in the root directory, build as any other rust application. (Training
Tracker is compiled into `hachimi.dll`; the installer no longer bundles a separate plugin
DLL.)

- **MSRV:** v1.77
- Features:
    - `compress_bin`: Compress the embedded binaries using zstd and decompress them during installation.
    - `training_tracker`: Embed and offer the optional Training Tracker plugin (requires the two extra files above).

# Vendored into HachimiRedux
This is a vendored fork of the upstream Hachimi installer, living as a workspace
member (`installer/`) of the HachimiRedux monorepo. It is built and released as part
of the repo's `create_release.yml` workflow — there is no separate installer repo,
tag, or release pipeline to coordinate. Bump `version` in `Cargo.toml` when the
installer changes meaningfully; it ships with the next HachimiRedux release.

# License
[MIT](LICENSE)

Upstream: forked from [teiosteppa/Installer](https://github.com/teiosteppa/Installer).
