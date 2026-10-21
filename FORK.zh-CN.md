English: [FORK.md](FORK.md) (the English edition is authoritative)

# FORK.md

这是 [WezTerm](https://github.com/wezterm/wezterm) 的个人 fork，由 [0x1b2c](https://github.com/0x1b2c) 维护。

## 概述

这是一个个人 fork，不是下游发行版：不更换品牌，也无意成为一个独立的产品。它会发布带版本号的 release，提供预编译的软件包和一个 Homebrew tap，因此无需自行构建即可安装（见[安装](#安装)）。它在上游 WezTerm 的 `main` 之上带有一组 patch：这些是日常使用中的修复与功能，涉及延迟、multiplexer 的正确性和工作流程的便利性，上游尚未提供。

每个功能位于各自的分支上，`master` 整合整组 patch。其中一个 patch 已提交给上游（见下文），其余仅存在于本 fork。除此之外的一切与上游保持一致。WezTerm 的一般用法见 <https://wezterm.org/>。

## 这个 fork 解决的问题

这些 patch 按主题分组，每一项都源于日常使用中的一个实际困扰，其中大多数围绕 WezTerm 经 SSH 使用的 multiplexer。每个条目先说明问题，实现修复的分支链接放在末尾。一个条目对应一个分支；分支内的各个 commit 是一个整体，这里不逐个列出。没有用户可见症状的构建与开发体验类 patch，按分支名列在最后一节。

### 在 mux 负载下，GUI 冻结或严重卡顿

- **重新连接打开了大量 tab 的 mux server 时，GUI 冻结。** mux 通道上的一次缓慢写入可能阻塞读取路径，使整个 session 死锁。修复方法是把 mux client 与 server 的分发拆成相互独立的 reader 与 writer task，通知则放在单独的 task 中处理。
  *分支：[mux-deadlock-fix](https://github.com/0x1b2c/wezterm/compare/main...mux-deadlock-fix)，已作为 [PR #7771](https://github.com/wezterm/wezterm/pull/7771) 提交给上游。*

- **打开了大量 tab 时调整窗口大小，GUI 冻结。** 每个 `TabResized` 事件都会触发对所有 pane 的完整重新同步，代价为 O(n²)，使界面卡死。修复方法是跳过多余的重新同步。
  *分支：[skip-resync-on-tab-resized](https://github.com/0x1b2c/wezterm/compare/ssh-proxy-error-msg...skip-resync-on-tab-resized)。*

- **程序快速更新终端标题时，tab 栏出现卡顿。** 频繁的 OSC 标题更新使每次更新都重新运行 `format-tab-title` 的整套处理，拖垮了渲染循环。修复方法是缓存这部分工作并在可能时直接跳过。
  *分支：[tab-title-perf](https://github.com/0x1b2c/wezterm/compare/objc-cargo-clippy-lint...tab-title-perf)。*

- **按键手感迟滞，因为渲染要等到下一个预定的帧。** 修复方法是在按键之后立即触发一次渲染轮询，降低感知到的输入延迟。
  *分支：[keystroke-render-poll](https://github.com/0x1b2c/wezterm/compare/fork-base...keystroke-render-poll)。*

### 一次声明窗口布局，按需生成

- **每次都手动建立相同的窗口和 tab，十分繁琐。** 本 fork 增加了 Window Presets：在配置中声明窗口布局，再通过 launcher 的命令行参数按需生成和关闭，使一组已知可用的工作环境始终只差一条命令。它还包括 `SpawnCommand` 的 `send_text` 字段，使 preset 中的 tab 可以在默认 shell 中自动运行一条命令。preset tab 的 `cwd` 可以以 `~/` 开头；省略 `cwd` 的 tab 会在前一个 tab 的目录中打开，因此一个项目的 preset 只需写一次目录。tab 也可以把第一个条目写成目录，并用 `cmd` 给出要在其 shell 中运行的一行命令。preset 窗口的标题以 `[<preset name>]` 开头，便于外部工具识别。
  *用法：[docs/fork/window-presets.md](docs/fork/window-presets.md)。分支：[window-presets](https://github.com/0x1b2c/wezterm/compare/session-experience...window-presets)。*

- **没有一个能一次关闭整个 mux 窗口的 keybinding action**，而重置布局需要它。新增了一个 Lua action，以原子方式结束一个 mux 窗口。
  *分支：[lua-kill-window](https://github.com/0x1b2c/wezterm/compare/tab-title-perf...lua-kill-window)。*

- **关闭一个窗口时，可能会断开其他窗口仍在使用的 mux domain。** 修复方法是只要还有窗口在使用该 domain，就保持它的连接。
  *分支：[session-experience](https://github.com/0x1b2c/wezterm/compare/lua-kill-window...session-experience)。*

### 在慢速连接上输入十分痛苦

- **在高延迟的连接上，每次按键都要经过一次完整的往返，字符才会出现在屏幕上。** 当 mux server 运行在一台已经休眠的 Mac 上时，情况尤其糟糕：macOS 的电源管理会限制休眠中的机器，每次往返从几毫秒拉长到几秒。本 fork 增加了 Local Input Mode，这是一种按 pane 设置的输入模式，在 client 端缓冲按键，只在提交时才把整行送往 PTY。它是在网络不佳时提供最低限度可用环境的便利工具，并不是完美的行编辑器。它是这组 patch 中最新的一个，在粘贴和完整按键捕获方面还有已知的边界情况尚未解决。
  *用法：[docs/fork/local-input.md](docs/fork/local-input.md)。分支：[local-input](https://github.com/0x1b2c/wezterm/compare/window-presets...local-input)。*

### mux pane 中的程序应当收到它们所请求的按键

- **在 mux pane 中，开启 kitty keyboard protocol 的程序无法识别某些按键，Ctrl+Shift 组合键则完全无法输入。** pane 的终端运行在 mux server 上，所以由它应答程序的请求，并报告该协议已开启；但按键是由 GUI 编码的，而 GUI 从未得知 pane 的键盘模式，因此一直发送传统编码。信任了这个应答的程序会丢弃传统序列，例如 Ctrl+Alt+E 的 `ESC ^E`，Shift+Enter 到达时则变成一次普通的 Enter。现在，mux server 会把 pane 的 kitty keyboard flags 连同 pane 的其余状态一起发送给它的 client，GUI 据此为 mux pane 编码按键，与它对本地 pane 的做法一致。只有当 GUI 与 mux server 两端的 `enable_kitty_keyboard` 都为 `true` 时，该协议才会生效。程序开启或关闭该协议，到 GUI 得知这一变化之间的短暂时刻里输入的按键，仍使用之前的编码。与传统按键不同，kitty 编码的按键在慢速连接上不会先于 server 在本地回显。它们不等待 mux server 的响应就直接发送，并与输入到该 pane 的其他按键、粘贴内容和文本保持顺序；出于同样的原因，mux pane 上的 `pane:send_text` 现在会在文本到达 mux server 之前返回，不再报告投递失败。
  *分支：[kitty-keyboard](https://github.com/0x1b2c/wezterm/compare/mux-startup...kitty-keyboard)。*

### 多台机器共享同一个 session

- **在另一台机器上使用同一个 mux session 之后，回到原来的机器，其窗口里显示的是按另一台机器排版的内容。** 每个 pane 在 mux server 上只有一个尺寸，由最后一个调整它的 client 决定；而 client 只在自己的窗口尺寸变化时才报告尺寸，所以内容会超出较小的窗口，或者让较大的窗口只填满一半，直到手动调整窗口大小。现在，尺寸跟随最近使用的那个 client，与 tmux 的 `window-size latest` 相同：当窗口获得焦点、收到按键、点击或滚动，或者切换 tab 时，如果各 pane 还不是它的尺寸，它就为自己当前的 tab 报告自己的尺寸。当两台机器同时显示这个 session 时，没有在使用的那一台显示的是另一台的尺寸，直到它再次被使用。

- **每个 pane 里的 ssh 突然找不到自己的密钥，并要求输入密钥的 passphrase。** pane 通过 mux server 维护的一条链接访问 ssh-agent，这条链接始终指向最近活跃的 client 的 agent。从某个 pane 内部启动的 GUI 会继承该 pane 的 `SSH_AUTH_SOCK`，也就是这条链接本身，并把它当作自己的 agent 报告；mux server 随后把链接指向了它自己，于是所有 pane 都失去了 agent，直到 session 重新启动。现在，mux server 会拒绝直接或经由其他链接绕回自身链接的目标，保留原来可用的目标，并记录 `[fix:agent-self-link]` 日志。
  *分支：[shared-session](https://github.com/0x1b2c/wezterm/compare/windows-cross-build...shared-session)。*

### mux server 应当可靠地启动

- **登录之后不久，GUI 以 "Connection refused" 退出，并留下一个没有窗口的 mux server 在运行。** 找不到 mux server 的 GUI 会自己启动一个，然后只给它大约两秒半的时间开始监听。刚登录时系统很忙，server 的启动加上它为了转入 daemon 而做的 re-exec 可能需要好几秒，于是 GUI 放弃并退出，而它启动的 server 过一会儿才就绪。现在，client 对自己刚启动的 server 会持续尝试连接，最长 15 秒。
  *分支：[mux-startup](https://github.com/0x1b2c/wezterm/compare/ssh-config-include...mux-startup)。*

### WezTerm 不应让 macOS 在后台忙个不停

- **在 macOS 26 及更高版本上，Spotlight 的索引服务每天因 WezTerm 写入数 GB 数据。** AppKit 会把一个应用的每个窗口都索引到 Spotlight，并且在任何窗口的标题变化时重新提交全部窗口，没有任何合并；终端的标题每秒变化好几次（shell 目录、程序标题、进度字符），而 Spotlight 会拒绝其中的每一次更新。现在，WezTerm 在启动时通过 AppKit 未公开的 `NSEnableWindowTabIndexing` 默认值关闭这项索引，并以注册默认值的方式设置，因此仍可用 `defaults write org.1b2c.wezterm NSEnableWindowTabIndexing -bool true` 将其重新打开。
  *分支：[spotlight-window-titles](https://github.com/0x1b2c/wezterm/compare/shared-session...spotlight-window-titles)。*

### SSH domain 应当像 `ssh` 一样查找主机

- **对于普通 `ssh` 能连上的主机，ssh domain 却报错 "failed to lookup address information"。** 这台主机定义在由 `Include ~/.ssh/config.local` 引入的文件中。WezTerm 不展开 `Include` 参数中的 `~`，并且把相对路径参数相对于包含该 `Include` 的文件所在目录解析，所以被引入的文件从未被读取，别名被当作字面的主机名。现在 WezTerm 遵循 `ssh_config(5)`：开头的 `~` 表示 home 目录；相对路径在用户配置文件中相对于 `~/.ssh` 解析，在系统配置文件中相对于 `/etc/ssh` 解析，无论 include 嵌套多深。glob 仍然有效。
  *分支：[ssh-config-include](https://github.com/0x1b2c/wezterm/compare/spotlight-window-titles...ssh-config-include)。*

### 失败应当自己说明原因

- **远程 SSH proxy command 失败时，得到的只是一条含糊的连接错误。** 修复方法是给出一条清楚的错误信息，说明究竟是什么失败了。
  *分支：[ssh-proxy-error-msg](https://github.com/0x1b2c/wezterm/compare/keystroke-render-poll...ssh-proxy-error-msg)。*

### 构建与开发体验

这些 patch 没有用户可见的症状，为完整起见列在这里。

- **[fork-base](https://github.com/0x1b2c/wezterm/compare/mux-deadlock-fix...fork-base)**
  本 fork 自己的文档（本文件、`docs/fork/`、README 的横幅），给 coding agent 的说明，以及本 fork 的 mux protocol 版本范围。它位于 `mux-deadlock-fix` 的正上方，而 `mux-deadlock-fix` 为了它的 pull request 直接建立在上游之上；这样它之上的每个功能分支都可以在引入功能的同一个 commit 中更新本 fork 的文档。

- **[macos-rerun-if-changed](https://github.com/0x1b2c/wezterm/compare/skip-resync-on-tab-resized...macos-rerun-if-changed)**
  修正构建脚本中 `rerun-if-changed` 的路径，使 macOS 构建不再做不必要的重新构建。

- ~~**[vendor-openssl-musl](https://github.com/0x1b2c/wezterm/compare/macos-rerun-if-changed...vendor-openssl-musl)**
  为 musl 目标 vendor OpenSSL，以支持静态交叉编译。~~
  已被上游取代：[wezterm#8028](https://github.com/wezterm/wezterm/pull/8028)（2026 年 8 月）做了相同的 `async_ossl` 修改，因此本 fork 的版本（2026 年 4 月）在 2026 年 10 月 rebase 这组 patch 时被舍弃。原来的 commit 保留为 tag [`fork/superseded/vendor-openssl-musl`](https://github.com/0x1b2c/wezterm/tree/fork/superseded/vendor-openssl-musl)。该分支现在只保留 `Cross.toml` 的配置，使 `cross` 能在其容器中构建静态的 Linux 二进制文件。

- **[objc-cargo-clippy-lint](https://github.com/0x1b2c/wezterm/compare/vendor-openssl-musl...objc-cargo-clippy-lint)**
  在 workspace 的 lint 配置中把 `objc` crate 的 `cargo-clippy` cfg 加入白名单，以消除多余的警告。

- **[macos-bundle-identity](https://github.com/0x1b2c/wezterm/compare/local-input...macos-bundle-identity)**
  加入带有构建与打包 recipe 的 `justfile`，并让 macOS 构建使用本 fork 自己的 bundle identifier 和稳定的 ad-hoc 签名（见 [Bundle identifier 与签名](#bundle-identifier-与签名)）。

- **[windows-cross-build](https://github.com/0x1b2c/wezterm/compare/macos-bundle-identity...windows-cross-build)**
  用一条命令交叉编译并打包 Windows（MinGW）构建，并把与特定机器相关的部署 recipe 移入不提交的 `justfile.local`。

- **[fork-release](https://github.com/0x1b2c/wezterm/compare/kitty-keyboard...fork-release)**
  发布相关的部分：`wezterm --version` 说明所属的 fork release 与所用的 mux protocol，本机构建报告它所源自的 release，GitHub Actions 构建各平台的软件包并在每个 fork release tag 上发布 release，以及这些 release 的安装说明（见[安装](#安装)与[版本号](#版本号)）。

除上述分支之外，这组 patch 还带有少量标记为 `[local]` 的 commit：针对 macOS `visibleFrame` 窗口几何的绕过方法、为定位崩溃而加的日志标记，以及对构建噪音的抑制。这些是实验性的或与特定环境相关的尝试，不是面向用户的功能，因此不逐个说明。

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

本 fork 的构建使用自己范围内的 mux protocol 编号（`0x1b2c_UUNN`，其中 `UU` 是上游的 protocol 版本，`NN` 是本 fork 的 revision），所以 fork 构建与上游 WezTerm 的 mux server 会互相拒绝连接，而不会交换双方都无法解码的消息。mux 连接的两端请运行同一个 fork 构建。

上游提供了 CI 配置，但没有简单的本地构建脚本。为此，本 fork 在 repo 根目录加入了一个 [`justfile`](justfile)。运行 `just` 可以列出可用的 recipe。

常用的构建 recipe：

- `just build-arm64`：原生 arm64 release 二进制文件（加上 `debug` 则为 debug 构建）
- `just bundle-arm64`：用 arm64 二进制文件组装 `target/WezTerm.app`
- `just build-x86`：`x86_64-apple-darwin` release 二进制文件
- `just build-linux`：静态的 `x86_64-unknown-linux-musl` 二进制文件（需要 `cross` 和 Docker）
- `just bundle-windows`：`x86_64-pc-windows-gnu`（MinGW）二进制文件，打包为 `target/WezTerm-windows.zip`（需要从 git 安装的 `cross` 和 Docker；要在另一台 Docker 主机上构建，从 `justfile.local` 导出 `WEZTERM_WINDOWS_DOCKER_CONTEXT`，当该主机看不到这个 checkout 时再加上 `WEZTERM_WINDOWS_CROSS_REMOTE=1`）

与特定机器相关的部署 recipe 放在不提交的 `justfile.local` 中，justfile 在它存在时会导入它；repo 只带有通用的构建与打包 recipe。

如果不想使用 `just` 构建，请按照上游的源代码构建说明 <https://wezterm.org/install/source.html> 操作，但从本 fork 的 `master` 分支构建，而不是上游的 `main`。

### Bundle identifier 与签名

本 fork 的 macOS 构建使用 bundle identifier `org.1b2c.wezterm`，而不是上游的 `com.github.wez.wezterm`。原因在于隐私权限。这些构建使用 ad-hoc 签名，没有开发者证书，而 macOS 把隐私授权（Accessibility、Full Disk Access 等）绑定在应用的 designated requirement 上。普通的 ad-hoc 签名把这个 requirement 绑定在二进制文件的 hash 上，所以每次重新构建都会丢失授权；这些 recipe 改为使用只指明 bundle identifier 的 requirement 来签名，以后的每个构建都满足它。

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

本 fork 是 rebase 到上游之上的一组 patch，rebase 会改写历史。为了让 release tag 保持意义，在两次上游同步之间，历史只追加不改写：release 之后，新的 commit 放在这组 patch 的顶部，把 commit 移入它所属的分支要等到下一次上游同步，那时整组 patch 本来就要 rebase。因此在同一个上游基线之内，每个 release tag 都是 `master` 的祖先。

一次上游同步会打破这种祖先关系：早先 release 的 tag 指向 rebase 之前的那组 patch，它们作为对当时所发布内容的记录仍然有效。每个 release 的说明都会写明它所基于的上游 commit。本地构建报告它所派生自的 release，例如 `1b2c-1.4.0-dev.3+abcd1234`；如果祖先中没有任何 release，则报告 `1b2c-dev+abcd1234`。

## 许可证与署名

许可证与上游 WezTerm 相同，上游所有的版权和署名声明都予以保留。见 [LICENSE.md](LICENSE.md)。
