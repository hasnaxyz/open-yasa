<div align="center">
	<sup>Special thanks to:</sup><br>

| <a href="https://go.warp.dev/yazi" target="_blank"><img alt="Warp sponsorship" width=350 src="https://github.com/warpdotdev/brand-assets/blob/main/Github/Sponsor/Warp-Github-LG-02.png"><br><b>Warp, built for coding with multiple AI agents</b><br><sup>Available for macOS, Linux and Windows</sup></a> |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |

</div>

## Open Yasa - Yazi With A Machine Layer

Open Yasa is Hasna's public fork of [Yazi](https://github.com/sxyazi/yazi). It keeps
Yazi's async terminal file manager core and adds an Open Machines-aware entry layer
for browsing across local and remote machines.

This fork intentionally stays close to upstream Yazi. Upstream is MIT-licensed;
the original license text is preserved in [LICENSE](LICENSE), and upstream
authorship remains credited in this repository's history and package metadata.

## Open Machines Integration

When Open Yasa starts without an explicit path, it opens a top-level machine
chooser before normal file browsing. Each entry shows the Open Machines slug,
friendly/display name when available, route kind, status, and platform.

- Local machine entries enter the current working directory through the real
  local filesystem, so local browsing remains the normal Yazi fast path.
- Remote machine entries enter `sftp://<machine-id>/...`; the SFTP service is
  resolved dynamically through the `machines route` command from
  [`@hasna/machines`](https://www.npmjs.com/package/@hasna/machines) when it is
  installed and configured.
- Remote file operations use Yazi's existing SFTP provider. Listing, reading,
  writing, copy, move, rename, delete, and symlink behavior are available only
  when the resolved SSH/SFTP route authenticates successfully.
- Remote SFTP host keys are checked against the user's `~/.ssh/known_hosts` by
  default. Add or verify host keys through your normal SSH/Open Machines trust
  flow before browsing a new remote machine; `no_cert_verify = true` remains an
  explicit opt-out for trusted private environments.
- If Open Machines is unavailable, Open Yasa falls back to a local machine entry
  and still works as a local terminal file manager.
- Unreachable or unauthenticated machines surface through Yazi's existing folder
  error state. Open Machines-generated SFTP services use a shorter connection
  timeout than static `vfs.toml` services.
- Adding and connecting machines is handled by the Open Machines CLI, for
  example `machines manifest add`, `machines setup --apply`, and
  `machines sync --apply`. The Open Yasa chooser re-reads topology on refresh,
  so newly connected machines appear without fork-specific private state.

Configuration default:

```toml
[open_yasa]
machines_layer = true
```

Run `yazi /some/path` to bypass the machine chooser and start directly in a local
or supported VFS path.

## Build And Install

```bash
cargo build --release --locked --bin yazi --bin ya
install -Dm755 target/release/yazi ~/.local/bin/open-yasa
install -Dm755 target/release/ya ~/.local/bin/open-yasa-ya
```

The upstream-compatible `yazi` and `ya` binaries are still built. Hasna installs
the fork under `open-yasa`/`open-yasa-ya` aliases to avoid replacing upstream
Yazi unless that is intentional.

GitHub release automation builds `open-yasa-*` draft/nightly artifacts. Store
publishing to Winget or Snap is disabled until Open Yasa has dedicated package
identities and release credentials.

## Upstream Sync

This repository tracks upstream Yazi through the read-only `upstream` remote:

```bash
git fetch upstream
git merge upstream/main
```

Keep fork-specific Open Yasa work in feature branches and PRs against
`hasnaxyz/open-yasa`; do not push to `upstream`.

## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi (means "duck") is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

💡 A new article explaining its internal workings: [Why is Yazi Fast?](https://yazi-rs.github.io/blog/why-is-yazi-fast)

- 🚀 **Full Asynchronous Support**: All I/O operations are asynchronous, CPU tasks are spread across multiple threads, making the most of available resources.
- 💪 **Powerful Async Task Scheduling and Management**: Provides real-time progress updates, task cancellation, and internal task priority assignment.
- 🖼️ **Built-in Support for Multiple Image Protocols**: Also integrated with Überzug++ and Chafa, covering almost all terminals.
- 🌟 **Built-in Code Highlighting and Image Decoding**: Combined with the pre-loading mechanism, greatly accelerates image and normal file loading.
- 🔌 **Concurrent Plugin System**: UI plugins (rewriting most of the UI), functional plugins, custom previewer/preloader/spotter/fetcher; Just some pieces of Lua.
- ☁️ **Virtual Filesystem**: Remote file management, custom search engines.
- 📡 **Data Distribution Service**: Built on a client-server architecture (no additional server process required), integrated with a Lua-based publish-subscribe model, achieving cross-instance communication and state persistence.
- 📦 **Package Manager**: Install plugins and themes with one command, keeping them up-to-date, or pin them to a specific version.
- 🧰 Integration with ripgrep, fd, fzf, zoxide
- 💫 Vim-like input/pick/confirm/which/notify component, auto-completion for cd paths
- 🏷️ Multi-Tab Support, Cross-directory selection, Scrollable Preview (for videos, PDFs, archives, code, directories, etc.)
- 🔄 Bulk Rename/Create, Archive Extraction, Visual Mode, File Chooser, [Git Integration](https://github.com/yazi-rs/plugins/tree/main/git.yazi), [Mount Manager](https://github.com/yazi-rs/plugins/tree/main/mount.yazi)
- 🎨 Theme System, Mouse Support, Drag and Drop, Trash Bin, Custom Layouts, CSI u, OSC 52
- ... and more!

https://github.com/sxyazi/yazi/assets/17523360/92ff23fa-0cd5-4f04-b387-894c12265cc7

## Project status

Public beta, can be used as a daily driver.

Yazi is currently in heavy development, expect breaking changes.

## Documentation

- Usage: https://yazi-rs.github.io/docs/installation
- Features: https://yazi-rs.github.io/features

## Discussion

- Discord Server (English mainly): https://discord.gg/qfADduSdJu
- Telegram Group (Chinese mainly): https://t.me/yazi_rs

## Image Preview

| Platform                                                                     | Protocol                               | Support                                |
| ---------------------------------------------------------------------------- | -------------------------------------- | -------------------------------------- |
| [kitty](https://github.com/kovidgoyal/kitty) (>= 0.28.0)                     | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [iTerm2](https://iterm2.com)                                                 | [Inline images protocol][iip]          | ✅ Built-in                            |
| [WezTerm](https://github.com/wez/wezterm)                                    | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Konsole](https://invent.kde.org/utilities/konsole)                          | [Kitty old protocol][kgp-old]          | ✅ Built-in                            |
| [foot](https://codeberg.org/dnkl/foot)                                       | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Ghostty](https://github.com/ghostty-org/ghostty)                            | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [Windows Terminal](https://github.com/microsoft/terminal) (>= v1.22.10352.0) | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [st with Sixel patch](https://github.com/bakkeby/st-flexipatch)              | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Warp](https://www.warp.dev) (macOS/Linux only)                              | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Tabby](https://github.com/Eugeny/tabby)                                     | [Inline images protocol][iip]          | ✅ Built-in                            |
| [VSCode](https://github.com/microsoft/vscode)                                | [Inline images protocol][iip]          | ✅ Built-in                            |
| [Rio](https://github.com/raphamorim/rio) (>= 0.3.9)                          | [Kitty unicode placeholders][kgp]      | ✅ Built-in                            |
| [Black Box](https://gitlab.gnome.org/raggesilver/blackbox)                   | [Sixel graphics format][sixel]         | ✅ Built-in                            |
| [Bobcat](https://github.com/ismail-yilmaz/Bobcat)                            | [Inline images protocol][iip]          | ✅ Built-in                            |
| X11 / Wayland                                                                | Window system protocol                 | ☑️ [Überzug++][ueberzug] required      |
| Fallback                                                                     | [ASCII art (Unicode block)][ascii-art] | ☑️ [Chafa][chafa] required (>= 1.16.0) |

See https://yazi-rs.github.io/docs/image-preview for details.

<!-- Protocols -->

[kgp]: https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders
[kgp-old]: https://github.com/sxyazi/yazi/blob/main/yazi-adapter/src/drivers/kgp_old.rs
[iip]: https://iterm2.com/documentation-images.html
[sixel]: https://www.vt100.net/docs/vt3xx-gp/chapter14.html
[ascii-art]: https://en.wikipedia.org/wiki/ASCII_art

<!-- Dependencies -->

[ueberzug]: https://github.com/jstkdng/ueberzugpp
[chafa]: https://hpjansson.org/chafa/

## Special Thanks

<img alt="RustRover logo" align="right" width="200" src="https://resources.jetbrains.com/storage/products/company/brand/logos/RustRover.svg">

Thanks to RustRover team for providing open-source licenses to support the maintenance of Yazi.

Active code contributors can contact @sxyazi to get a license (if any are still available).

## License

Yazi is MIT-licensed. For more information check the [LICENSE](LICENSE) file.
