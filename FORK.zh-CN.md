English: [FORK.md](FORK.md) (the English edition is authoritative)

# FORK.md

这是 [WezTerm](https://github.com/wezterm/wezterm) 的一个 fork，由 [0x1b2c](https://github.com/0x1b2c) 维护。

## 概述

它不是下游发行版：不更换品牌，也不打算成为独立的产品。它会发布带版本号的 release，提供预编译的软件包和一个 Homebrew tap，因此无需自行构建即可安装（见[安装](#安装)）。它在上游 WezTerm 的 `main` 之上带有一组 patch：这些是日常使用中的修复与功能，涉及延迟、multiplexer 的正确性和工作流程的便利性，上游尚未提供。

每个功能位于各自的分支上，`master` 整合整组 patch。其中一个 patch 已提交给上游（见下文），其余仅存在于本 fork。除此之外的一切与上游保持一致。WezTerm 的一般用法见 <https://wezterm.org/>。

使用本 fork 时遇到问题，请到 <https://github.com/0x1b2c/wezterm/issues> 报告，不要报给上游 WezTerm：问题可能出在本 fork 的某个 patch 上。

## 这个 fork 解决的问题

这些 patch 按主题分组，每一项都源于日常使用中的一个实际困扰，其中大多数与经 SSH 使用 WezTerm 的 multiplexer 有关。每个条目先说明问题，再说明装上本 fork 之后有什么变化、使用时需要知道什么；条目末尾链接到承载该 patch 的分支，修复的具体做法见各分支的 commit。构建与开发体验方面的 patch 不改变用户看到的行为，按分支名列在最后一节。

### 在 mux 负载下，GUI 冻结或严重卡顿

- **重新连接打开了大量 tab 的 mux server 时，GUI 冻结。** 现在打开几个窗口、共约五十个 tab 时，重新连接也不会冻结。
  *分支：[mux-deadlock-fix](https://github.com/0x1b2c/wezterm/compare/main...mux-deadlock-fix)，已作为 [PR #7771](https://github.com/wezterm/wezterm/pull/7771) 提交给上游。*

- **打开了大量 tab 时调整窗口大小，GUI 冻结。** 现在打开几个窗口、共约五十个 tab 时，调整窗口大小，界面也能及时响应。
  *分支：[skip-resync-on-tab-resized](https://github.com/0x1b2c/wezterm/compare/ssh-proxy-error-msg...skip-resync-on-tab-resized)。*

- **程序频繁更新终端标题时，tab 栏出现卡顿。** 即使 Codex 这类程序每秒改写标题许多次，tab 栏也能保持流畅。一个程序在五分之一秒内设置三次以上标题时，它的 tab 直接显示程序设置的标题，不再调用 `format-tab-title` 处理函数，直到标题停止变化满一秒为止。
  *分支：[tab-title-perf](https://github.com/0x1b2c/wezterm/compare/build-tooling...tab-title-perf)。*

- **按键反应迟缓。** 现在输入的字符会立即显示，不必等到下一个预定的帧。
  *分支：[keystroke-render-poll](https://github.com/0x1b2c/wezterm/compare/fork-base...keystroke-render-poll)。*

### 一次声明窗口布局，按需生成

- **每次都手动建立相同的窗口和 tab，十分繁琐。** 本 fork 增加了 Window Presets：在配置中声明窗口布局，再从 launcher 按需生成或关闭，配置好的整套窗口随时只需一条命令即可打开；`startup_windows` 可以在 mux server 启动时打开指定的 preset。每个 tab 可以指定目录（可以以 `~/` 开头）以及要在其 shell 中运行的一行命令；不指定目录的 tab 在前一个 tab 的目录中打开，因此一个项目的 preset 只需写一次目录。简写 `{ '~/project', cmd = 'make watch' }` 等同于 `{ cwd = '~/project', send_text = 'make watch\n' }`。preset 窗口的标题以 `[<preset name>]` 开头，便于外部工具识别。preset 由 mux server 打开，因此这些设置要在 mux server 也运行本 fork 时才会生效。
  *用法：[docs/fork/window-presets.md](docs/fork/window-presets.md)。分支：[window-presets](https://github.com/0x1b2c/wezterm/compare/session-experience...window-presets)。*

- **没有一个能一次关闭整个 mux 窗口的 keybinding action**，而重置布局需要它。现在可以用 `wezterm.mux.kill_window(mux_window)` 一步关闭一个 mux 窗口及其全部 tab，推荐绑定到 `Cmd+Shift+W`：

  ```lua
  { key = 'w', mods = 'CMD|SHIFT', action = wezterm.action_callback(function(window)
    wezterm.mux.kill_window(window:mux_window())
  end) },
  ```
  *分支：[lua-kill-window](https://github.com/0x1b2c/wezterm/compare/tab-title-perf...lua-kill-window)。*

- **关闭一个窗口时，可能会 detach 其他窗口仍在使用的 mux domain。** 现在只要还有窗口在使用该 domain，它就保持 attach。
  *分支：[session-experience](https://github.com/0x1b2c/wezterm/compare/lua-kill-window...session-experience)。*

### 在慢速连接上输入十分痛苦

- **在高延迟的连接上，每次按键都要经过一次完整的往返，字符才会出现在屏幕上。** 当 mux server 运行在一台已经休眠的 Mac 上时情况尤其糟糕，每次往返会从几毫秒拉长到几秒。Local Input Mode 按 pane 开启，开启后可以在本地输入并编辑一整行，提交时才把这一行发送出去。它的用途是在网络不佳时维持一个最低限度可用的环境，并不是一个完整的行编辑器。已知的限制有：持续重绘输入区域的程序（例如 Claude Code）会覆盖本地显示的文字；`Cmd+V` 粘贴的内容直接发送给远程程序；在本地输入期间，Tab 补全、历史搜索等 readline 功能不可用；向普通 shell 提交多行文字时，shell 会逐行执行。用法说明中有这些限制的详细描述。
  *用法：[docs/fork/local-input.md](docs/fork/local-input.md)。分支：[local-input](https://github.com/0x1b2c/wezterm/compare/window-presets...local-input)。*

### 完整支持 Kitty 键盘协议

- **程序运行在 mux server 中时（例如把默认 domain 设为一个 unix domain，或者通过 SSH domain 连接到远程机器），一旦开启 Kitty 键盘协议就会漏掉按键，Ctrl+Shift 组合键则完全无法输入。** 上游 WezTerm 只在不经过 mux server 时才完整支持这个协议；本 fork 让运行在 mux server 中的程序同样可以使用完整的 Kitty 键盘协议：Ctrl+Shift 组合键、与 Enter 区分开的 Shift+Enter、与 Tab 区分开的 Ctrl+I，以及 Ctrl+Alt 加字母的快捷键，都能送达程序。使用条件是 GUI 与 mux server 两端的配置中 `enable_kitty_keyboard` 都为 `true`，并且两端都运行 protocol 版本为 46.3 或更新的本 fork（见[版本号](#版本号)）。在慢速连接上，以 Kitty 键盘协议编码发送的按键不会先于 server 在本地回显。对运行在 mux server 中的 pane 调用 `pane:send_text` 时，它会在文本到达 mux server 之前返回，不再报告投递失败。
  *分支：[kitty-keyboard](https://github.com/0x1b2c/wezterm/compare/mux-startup...kitty-keyboard)。*

### 多台机器共享同一个 session

- **在另一台机器上使用同一个 mux session 之后，回到原来的机器，窗口里显示的仍是按另一台机器的屏幕大小排版的内容。** 现在 pane 的尺寸跟随正在使用的那台机器，与 tmux 的 `window-size latest` 相同：窗口一旦获得焦点、收到按键、点击或滚动，或者切换 tab，其中的 pane 就会调整到适合这个窗口的尺寸。两台机器同时显示这个 session 时，没有在使用的那一台显示另一台的尺寸，直到再次使用它为止。

- **每个 pane 里的 ssh 突然找不到密钥，并要求输入密钥的 passphrase。** 这种情况发生在从某个 pane 内部启动 WezTerm GUI 之后。现在以这种方式启动 GUI，不会再使其他 pane 失去 ssh-agent。
  *分支：[shared-session](https://github.com/0x1b2c/wezterm/compare/windows-cross-build...shared-session)。*

### mux server 应当可靠地启动

- **mux server 启动较慢时，GUI 以 "Connection refused" 退出，并留下一个没有窗口的 mux server 在运行。** 现在 GUI 会为自己刚启动的 mux server 最多等待 15 秒，即使 mux server 启动得慢，GUI 也能正常打开。
  *分支：[mux-startup](https://github.com/0x1b2c/wezterm/compare/ssh-config-include...mux-startup)。*

### WezTerm 不应让 macOS 在后台忙个不停

- **在 macOS 26 及更高版本上，Spotlight 的索引服务每天因 WezTerm 写入数 GB 数据。** 终端的窗口标题每秒变化好几次，而 macOS 每次都会把所有窗口重新提交给 Spotlight。现在 WezTerm 关闭了这项索引。如需重新打开，运行 `defaults write org.1b2c.wezterm NSEnableWindowTabIndexing -bool true`。
  *分支：[spotlight-window-titles](https://github.com/0x1b2c/wezterm/compare/shared-session...spotlight-window-titles)。*

### SSH domain 应当像 `ssh` 一样查找主机

- **对于普通 `ssh` 能连上的主机，ssh domain 却报错 "failed to lookup address information"**，原因是这台主机定义在通过 `Include` 引入的文件中，例如 `Include ~/.ssh/config.local`。现在 ssh domain 按 `ssh` 的方式读取被引入的文件（`ssh_config(5)`）：开头的 `~` 表示 home 目录；相对路径在用户配置中相对于 `~/.ssh` 查找，在系统配置中相对于 `/etc/ssh` 查找，无论 include 嵌套多深。glob 仍然有效。
  *分支：[ssh-config-include](https://github.com/0x1b2c/wezterm/compare/spotlight-window-titles...ssh-config-include)。*

### 失败应当自己说明原因

- **远程 SSH proxy command 失败时，得到的只是一条含糊的连接错误。** 现在错误信息会说明究竟是什么失败了。
  *分支：[ssh-proxy-error-msg](https://github.com/0x1b2c/wezterm/compare/keystroke-render-poll...ssh-proxy-error-msg)。*

### 构建与开发体验

这些 patch 不改变用户看到的行为，为完整起见列在这里。

- **[fork-base](https://github.com/0x1b2c/wezterm/compare/mux-deadlock-fix...fork-base)**
  本 fork 自己的文档（本文件、`docs/fork/`、README 的横幅），给 coding agent 的说明，以及本 fork 的 mux protocol 版本范围。它位于 `mux-deadlock-fix` 的正上方，而 `mux-deadlock-fix` 为了它的 pull request 直接建立在上游之上；这样它之上的每个功能分支都可以在引入功能的同一个 commit 中更新本 fork 的文档。

- **[macos-rerun-if-changed](https://github.com/0x1b2c/wezterm/compare/skip-resync-on-tab-resized...macos-rerun-if-changed)**
  修正构建脚本中 `rerun-if-changed` 的路径，使 macOS 构建不再做不必要的重新构建。

- ~~**[vendor-openssl-musl](https://github.com/0x1b2c/wezterm/compare/macos-rerun-if-changed...vendor-openssl-musl)**
  为 musl 目标 vendor OpenSSL，以支持静态交叉编译。~~
  已被上游取代：[wezterm#8028](https://github.com/wezterm/wezterm/pull/8028)（2026 年 8 月）做了相同的 `async_ossl` 修改，因此本 fork 的版本（2026 年 4 月）在 2026 年 10 月 rebase 这组 patch 时被舍弃。原来的 commit 保留为 tag [`fork/superseded/vendor-openssl-musl`](https://github.com/0x1b2c/wezterm/tree/fork/superseded/vendor-openssl-musl)。该分支现在只保留 `Cross.toml` 的配置，使 `cross` 能在其容器中构建静态的 Linux 二进制文件。

- **[objc-cargo-clippy-lint](https://github.com/0x1b2c/wezterm/compare/vendor-openssl-musl...objc-cargo-clippy-lint)**
  在 workspace 的 lint 配置中把 `objc` crate 的 `cargo-clippy` cfg 加入白名单，以消除多余的警告。

- **[build-tooling](https://github.com/0x1b2c/wezterm/compare/objc-cargo-clippy-lint...build-tooling)**
  加入带有构建、打包与签名 recipe 的 `justfile`，CI 打包 macOS 版本时也用它签名；并屏蔽 macOS 构建为空目标文件输出的「has no symbols」警告。

- **[macos-bundle-identity](https://github.com/0x1b2c/wezterm/compare/local-input...macos-bundle-identity)**
  让 macOS 构建使用本 fork 自己的 bundle identifier 和稳定的 ad-hoc 签名（见 [Bundle identifier 与签名](#bundle-identifier-与签名)）。

- **[windows-cross-build](https://github.com/0x1b2c/wezterm/compare/macos-bundle-identity...windows-cross-build)**
  用一条命令交叉编译并打包 Windows（MinGW）构建，并把与特定机器相关的部署 recipe 移入不提交的 `justfile.local`。

- **[fork-release](https://github.com/0x1b2c/wezterm/compare/kitty-keyboard...fork-release)**
  发布相关的部分：`wezterm --version` 说明所属的 fork release 与所用的 mux protocol，本机构建报告它所源自的 release，GitHub Actions 构建各平台的软件包并在每个 fork release tag 上发布 release，以及这些 release 的安装说明（见[安装](#安装)与[版本号](#版本号)）。

## 安装

每个 release 都发布在 [GitHub Releases](https://github.com/0x1b2c/wezterm/releases) 页面上，每个软件包旁边都有一个 `.sha256` 文件。

### macOS：Homebrew

```sh
brew install --cask 0x1b2c/wezterm/wezterm-1b2c
```

这个 cask 会安装 `WezTerm.app`，并把 `wezterm`、`wezterm-gui`、`wezterm-mux-server` 和 `strip-ansi-escapes` 链接到 Homebrew 的 `bin` 中。它与官方的 `wezterm` 和 `wezterm@nightly` cask 冲突，因为它们链接的是相同的名字，所以安装其中一个之前要先卸载另一个。这些软件包使用 ad-hoc 签名，没有经过 notarization；cask 在安装之后会清除应用的 quarantine 属性。

### macOS：直接下载

下载 `WezTerm-1b2c-<version>-macos-arm64.zip`（Apple silicon）或 `WezTerm-1b2c-<version>-macos-x86_64.zip`（Intel）并解压。由于应用没有经过 notarization，只要下载文件带有 quarantine 属性，macOS 就拒绝打开它。先清除该属性，再把应用移到 `/Applications`：

```sh
xattr -cr WezTerm.app
```

### Windows

下载 `WezTerm-1b2c-<version>-windows-x86_64.zip`，解压后运行 `wezterm-gui.exe`。这些可执行文件没有签名，所以 Windows SmartScreen 可能显示 "Windows protected your PC"。选择 "More info"，再选择 "Run anyway"。若要避免为每个可执行文件都出现这个提示，可以在解压之前打开 zip 的 Properties，勾选 "Unblock"。

### Linux

`wezterm-1b2c-<version>-linux-x86_64-musl.tar.gz` 包含静态链接的 `wezterm` 和 `wezterm-mux-server` 二进制文件，用于命令行，以及在远程机器上运行 mux server。Linux 没有 GUI 软件包；请从源代码构建（见[构建](#构建)）。

## 构建

本 fork 修改了 mux protocol，因此它的构建与上游 WezTerm 的构建无法互通。本 fork 在自己的范围内为 protocol 版本编号（`0x1b2c_UUNN`，其中 `UU` 是上游的 protocol 版本，`NN` 是本 fork 的 revision），双方因此都能认出对方不兼容，直接拒绝连接并给出明确的提示，而不是连上之后出错。mux 连接的两端请运行 protocol 版本相同的 fork 构建。

上游提供了 CI 配置，但没有简单的本地构建脚本。为此，本 fork 在 repo 根目录加入了一个 [`justfile`](justfile)。运行 `just` 可以列出可用的 recipe。

常用的构建 recipe：

- `just build-arm64`：原生 arm64 release 二进制文件（加上 `debug` 则为 debug 构建）
- `just bundle-arm64`：用 arm64 二进制文件组装 `target/WezTerm.app`
- `just build-x86`：`x86_64-apple-darwin` release 二进制文件
- `just build-linux`：静态的 `x86_64-unknown-linux-musl` 二进制文件（需要 `cross` 和 Docker；使用与 `bundle-windows` 相同的 Docker 主机设置）
- `just bundle-windows`：`x86_64-pc-windows-gnu`（MinGW）二进制文件，打包为 `target/WezTerm-windows.zip`（需要从 git 安装的 `cross` 和 Docker；要在另一台 Docker 主机上构建，从 `justfile.local` 导出 `WEZTERM_CROSS_DOCKER_CONTEXT`，当该主机看不到这个 checkout 时再加上 `WEZTERM_CROSS_REMOTE=1`）

与特定机器相关的部署 recipe 放在不提交的 `justfile.local` 中，justfile 在它存在时会导入它；repo 只带有通用的构建与打包 recipe。

如果不想使用 `just` 构建，请按照上游的源代码构建说明 <https://wezterm.org/install/source.html> 操作，但从本 fork 的 `master` 分支构建，而不是上游的 `main`。

### Bundle identifier 与签名

本 fork 的 macOS 构建使用 bundle identifier `org.1b2c.wezterm`，而不是上游的 `com.github.wez.wezterm`。原因在于隐私权限。这些构建使用 ad-hoc 签名，没有开发者证书，而 macOS 把隐私授权（Accessibility、Full Disk Access 等）绑定在应用的 designated requirement 上。普通的 ad-hoc 签名把这个 requirement 绑定在二进制文件的 hash 上，每次重新构建都会丢失授权。因此这些 recipe 签名时，requirement 中只写明 bundle identifier，以后每次安装新版本都不必重新授权。

macOS 为每个 bundle identifier 保存一份权限记录。如果本 fork 沿用上游的 identifier，官方构建和 fork 构建就会争用这唯一的一份记录：给其中一个授予权限会替换另一个的权限，在两者之间切换时会反复要求重新授权。使用不同的 identifier，两种构建的权限就互不干扰。（共用 identifier 还会使官方构建满足本 fork 的 requirement，因为该 requirement 只指明 identifier。）这并不表示本 fork 主张作者身份：应用名称、版权声明和许可证都没有改变，WezTerm 仍然是 Wez Furlong 的作品。

## 版本号

本 fork 的 release 以 `MAJOR.MINOR.PATCH` 编号，以 `1b2c-<version>` 为名发布，例如 `1b2c-1.4.0`。这个名字既是 git tag 和 GitHub release 的名字，也是 `wezterm --version` 的输出，因此在任何地方看到的版本号都直接指向对应的 tag。上游自己的版本号是日期，所以两者不会混淆。

- 当 release 带来一批新功能时，**MINOR** 加一，PATCH 重新从 0 开始。
- 没有新功能的 release，**PATCH** 加一，通常是只让本 fork 追上上游的 release。
- **MAJOR** 标志一个里程碑，很少改变，由刻意的选择决定，而不是依据某条规则。

第一个 release 是 1.4.0 而不是 1.0.0，因为在它之前已经有四批功能：2026 年 3 月下旬到 4 月上旬的 mux 与延迟修复，4 月下旬的 tab 与窗口管理工作和 Window Presets，5 月的 Local Input Mode，以及 10 月的 preset 窗口稳定标识。它们从未单独发布过，所以 1.1 到 1.3 没有 tag。

`wezterm --version` 还会打印 mux **protocol version**，例如 `wezterm 1b2c-1.4.0 (protocol 46.3)`：先是上游的 protocol 版本，再是本 fork 自己的 protocol revision。protocol 版本与 release 版本相互独立：只有两个数字都一致时，GUI 与 mux server 才能互相通信（见[构建](#构建)）。只要某个 release 改变了它，release notes 就会写明，因为所有共享 mux session 的机器随后都必须一起升级。

脚本看到的仍是上游的格式：Lua 中的 `wezterm.version`、`TERM_PROGRAM_VERSION` 以及更新检查，仍然报告基于日期的版本号，所以针对上游版本号编写的比较逻辑可以继续使用。本 fork 的版本号以 [`wezterm.fork_version`](docs/config/lua/wezterm/fork_version.md) 的形式提供给配置使用。

### Release tag 与历史

本 fork 是叠在上游代码之上的一组 patch。每次同步上游时，都要通过 rebase 把这组 patch 重新应用到上游最新的代码上。rebase 会生成新的 commit，原有的 commit 不再出现在 `master` 的历史中。

为了让 release tag 始终能在 `master` 的历史中找到，两次同步之间只在这组 patch 的末尾追加 commit，不改动已有的 commit。本应归入下层分支的改动，也先追加在末尾，到下一次同步时再随整组 patch 一起移入对应的分支。因此，同一次同步之后发布的所有 release，其 tag 都在 `master` 的历史中。

同步之后，此前 release 的 tag 仍指向同步前的那组 commit，不再位于 `master` 的历史中，但它们依然准确记录了当时发布的内容。每个 release 的说明都写明它基于上游的哪个 commit。本机构建的版本号会注明它是在哪个 release 之后构建的：`1b2c-1.4.0-dev.3+abcd1234` 表示在 1.4.0 之后又有 3 个 commit，构建自 commit `abcd1234`。

## 许可证与署名

许可证与上游 WezTerm 相同，上游所有的版权和署名声明都予以保留。见 [LICENSE.md](LICENSE.md)。
