<img align="left" width="80" height="80" src="apps/hachimi/assets/icon.png">

# HachimiRedux

[English](README.md) | 简体中文 | [繁體中文](README-zh_tw.md) | [Español (España)](README-es_es.md) | [Español (Latinoamérica)](README-es_419.md) | [Français](README-fr_fr.md) | [Português (Brasil)](README-pt_br.md) | [Português (Portugal)](README-pt_pt.md)

面向 UM:PD 的游戏增强与翻译 Mod。HachimiRedux 是 Hachimi 的一个分支，内置了游戏内的养成追踪插件，并重写了原生插件 SDK。

<img height="400" src="apps/hachimi/assets/screenshot-2.png">

## 目录

- [请不要链接到本仓库或 Hachimi 的网站](#️-请不要链接到本仓库或-hachimi-的网站)
- [与上游 Hachimi 插件不兼容](#️-与上游-hachimi-插件不兼容)
- [特性](#特性)
- [安装](#安装)
  - [使用安装器安装（推荐）](#使用安装器安装推荐)
  - [从源码构建（进阶）](#从源码构建进阶)
- [故障排查](#故障排查)
- [特别鸣谢](#特别鸣谢)
- [许可证](#许可证)

# ⚠️ 请不要链接到本仓库或 Hachimi 的网站
我们理解你想帮助大家安装 Hachimi 并获得更好的游戏体验。然而，本项目本质上违反了游戏的服务条款，游戏开发者一旦知道它的存在，几乎肯定会希望将其铲除。

在你自己管理的聊天服务中以及通过私信分享是可以的，但我们恳请你不要在面向公众的网站上分享本项目的链接，也不要分享任何相关工具的链接。

或者你也可以照样分享，把它给那为数不多的 Hachimi 用户毁掉。这取决于你。

### 如果你无论如何都要分享
你想怎么做都行，但我们恳请你尽量将游戏标注为 “UM:PD” 或 “The Honse Game”，而不是游戏的真实名称，以避免被搜索引擎抓取。

# ⚠️ 与上游 Hachimi 插件不兼容
本分支自带其专属的原生插件 API（host API v9）。**为上游 Hachimi 构建的插件无法在 HachimiRedux 上使用**，而这里分发的养成追踪插件也无法在上游 Hachimi 上加载。请优先使用从本仓库构建的 DLL，并将它们搭配使用。混用不同的构建可能导致加载失败或游戏崩溃。

## 旧版插件兼容（可选，受限）
无清单、采用旧版 ABI 的插件（例如上游 Hachimi 的数据导出器）可以通过一条**可选的兼容路径**加载。除了 `load_libraries` 之外，还需在 `config.json` 中的 `legacy_libraries` 白名单里列出该 DLL：

```json
{
  "windows": {
    "load_libraries": ["some_legacy_plugin.dll"],
    "legacy_libraries": ["some_legacy_plugin.dll"]
  }
}
```

旧版插件只需导出 `hachimi_init`；宿主会跳过其通常的清单/ABI 检查，并基于信任加载它。该支持是**受限且不受官方支持的**：

- 该插件必须**仅依赖宿主 API 的稳定 vtable 前缀**。超出此范围的任何用法都属于未定义行为，可能导致游戏崩溃。
- 宿主**无法验证、追踪或卸载**旧版插件及其 IL2CPP 钩子。该 DLL 会在整个进程生命周期内保持映射。
- 每当一个插件通过此路径加载时，都会记录一条警告。
- `legacy_libraries` 中的条目也必须出现在 `load_libraries` 中。

如有疑问，请针对本仓库（host API v9）重新编译插件，而不要依赖旧版路径。

# 特性
- **高质量翻译：** Hachimi 提供了先进的翻译功能，让译文更加自然（复数形式、序数词等），并避免给 UI 引入错乱。它还支持翻译游戏中的大多数组件，无需手动打补丁替换资源！

    支持的组件：
    - UI 文本
    - master.mdb（技能名称、技能描述等）
    - 比赛剧情
    - 主线剧情/主页对话
    - 歌词
    - 纹理替换
    - 精灵图集替换

    此外，Hachimi 并非只为某一种语言提供翻译功能；它在设计上可针对任何语言完全自定义。

- **轻松上手：** 即插即用。所有设置都在游戏内完成，无需任何外部程序。
- **翻译自动更新：** 内置的翻译更新器让你照常游玩的同时进行更新，完成后在游戏内重新加载，无需重启！
- **内置图形界面：** 自带配置编辑器，让你无需退出游戏即可修改设置！
- **图形设置：** 你可以调整游戏的图形设置，以充分发挥设备性能，例如解锁 FPS 和分辨率缩放。
- **仅限 Windows：** 专为游戏的 Windows（Steam）版本打造。**HachimiRedux 出于自身选择不支持 Android** —— 它只专注于 Windows 客户端，没有计划添加或维护 Android 版本。

# 安装

安装 HachimiRedux 最简单的方式是使用 [Releases 页面](https://github.com/jalbarrang/hachimi-redux/releases)上的**安装器**：它会为你配置好核心 Mod 和可选的 Training Tracker 插件，无需手动复制文件或编辑 JSON。如果你更愿意自行构建，请参阅[从源码构建](#从源码构建进阶)。

HachimiRedux 是核心 Mod（以 `cri_mana_vpx.dll` 的形式加载）；**Training Tracker（养成追踪器）** 是由核心 Mod 加载的可选插件 DLL。两者来自同一次构建。

游戏目录即 Steam 安装文件夹，例如
`C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby`。

## 使用安装器安装（推荐）

1. 从 [Releases 页面](https://github.com/jalbarrang/hachimi-redux/releases)下载最新的 `hachimi_installer.exe`。
2. 运行它。安装器会自动检测你的 Steam 游戏目录；如果检测不到，请手动选择（默认路径见上方）。
3. 选择你的语言。若需要游戏内的 Training Tracker，请保持勾选 **“Install Training Tracker plugin”** 复选框（默认已勾选）。
4. 点击 **Install**。安装器会备份原始的 `cri_mana_vpx.dll`、安装 Mod，并为你创建 `config.json`。
5. 启动游戏。按下菜单键 —— 默认是**右方向键** —— 打开游戏内 UI。

以后要更新或移除 HachimiRedux，只需再次运行安装器即可（它提供卸载选项）。

## 从源码构建（进阶）

本仓库是一个 Cargo workspace。在仓库根目录下：

```sh
# 核心 Mod
cargo build --release -p hachimi                    # -> target/release/hachimi.dll
# Training Tracker 插件
cargo build --release -p hachimi-training-tracker   # -> target/release/hachimi_training_tracker.dll
```

## 安装 HachimiRedux（核心）

游戏通过渲染器 DLL `cri_mana_vpx.dll` 加载该 Mod。

1. 在游戏目录中，将原始的 `cri_mana_vpx.dll` 备份为 `cri_mana_vpx.dll.backup`（只做一次 —— 之后切勿覆盖该备份）。
2. 将 `target/release/hachimi.dll` 复制到游戏目录，并重命名为 `cri_mana_vpx.dll`。
3. 启动游戏。按下菜单键 —— 默认是**右方向键** —— 打开游戏内 UI。启动闪屏会显示当前按键，你也可以在游戏内图形界面中重新绑定它。

Mod 设置保存在游戏数据目录中的 `config.json` 内，该目录是**游戏目录下的 `hachimi` 子文件夹**（例如 `…\UmamusumePrettyDerby\hachimi\config.json`）。它会由安装器/首次启动时自动创建；其余所有内容都通过游戏内图形界面进行配置。

## 安装 Training Tracker 插件

插件是核心 Mod 在启动时从游戏目录根部加载的原生 DLL。

1. 先安装 HachimiRedux 核心（见上文）。
2. 将 `target/release/hachimi_training_tracker.dll` 复制到游戏目录根部（与 `cri_mana_vpx.dll` 同一文件夹）。注意：插件 DLL 放在游戏**根目录**，而 `config.json` 位于 `hachimi` 子文件夹中。
3. 在 `config.json`（`<game_dir>\hachimi\config.json`）的 `load_libraries` 列表中添加该 DLL：

   ```json
   {
     "windows": {
       "load_libraries": ["hachimi_training_tracker.dll"]
     }
   }
   ```
4. 启动游戏。追踪器会作为 Plugins 标签页中的一个页面出现，同时也作为一个浮动的覆盖面板。关于插件的工作原理，请参阅 [docs/plugin-sdk.md](docs/plugin-sdk.md)。

## 自动部署（Windows，从源码）

在仓库根目录下，辅助脚本会构建并将两个 DLL 复制到游戏目录：

```powershell
.\scripts\deploy-windows.ps1 -Build
```

如果游戏不在默认的 Steam 路径，可覆盖游戏文件夹：

```powershell
$env:HACHIMI_GAME_DIR = "D:\path\to\UmamusumePrettyDerby"
.\scripts\deploy-windows.ps1 -Build
```

该脚本会将 `hachimi.dll` → `cri_mana_vpx.dll` 以及 training tracker DLL 复制到游戏目录，且绝不会修改 `cri_mana_vpx.dll.backup`。

# 故障排查

## 游戏启动时崩溃／行为异常

到目前为止最常见的原因是在游戏文件夹中**堆叠了多个游戏 Mod 或 DLL 注入器**。每一个都会挂钩游戏的渲染/运行时，互相争抢。HachimiRedux 会在游戏内（一条通知 + `hachimi.log`）对此发出警告，安装器也会在安装前发出警告，但你必须自己移除其他的：

- **只**保留 HachimiRedux：`cri_mana_vpx.dll` 以及任何由 HachimiRedux 构建的插件（例如 `hachimi_training_tracker.dll`）。
- 从游戏文件夹中移除其他覆盖层/注入器，例如不该出现的代理加载器 DLL（`version.dll`、`winhttp.dll`、`dxgi.dll`、`d3d11.dll`、`dinput8.dll` 等）以及具名覆盖层（`horseACT.dll`、`heaven_overlay.dll` 等）。
- **只有由 HachimiRedux 构建的插件**才应放入 `load_libraries`。不要在其中添加第三方覆盖层 —— 它们不是 HachimiRedux 插件，会被拒绝（并附带游戏内提示），或可能导致游戏崩溃。

## 各文件位置

- `cri_mana_vpx.dll` 和插件 DLL：游戏**根**目录。
- `config.json` 及其他 Mod 数据：游戏目录的 **`hachimi` 子文件夹**（`<game_dir>\hachimi\config.json`）。
- Mod 日志：游戏根目录下的 `hachimi.log`（在 `config.json` 中启用 `enable_file_logging`）。
- 游戏日志：`%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log`。

## 收集诊断信息

- 游戏内：打开菜单（默认右方向键）→ **Config** → **Save diagnostics report**。这会在游戏文件夹中写入 `hachimi_diagnostics.txt`。
- 安装器：运行 `installer collect-logs`，将 `config.json`、`hachimi.log` 以及一份冲突报告收集到 `%TEMP%\hachimi_diagnostics`。

# 特别鸣谢

HachimiRedux 是一个建立在以下项目工作之上的分支：

- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi) —— 本项目所基于的原始项目。如果你对原始项目感兴趣，欢迎加入[它的 Discord 服务器](https://discord.gg/YjBgmuqqYr)。
- [Hachimi Edge](https://github.com/kairusds/Hachimi-Edge) —— HachimiRedux 所延续的、专注于 Windows/Steam 的分支。

这些项目又反过来成为了 Hachimi 开发的基础；没有它们，Hachimi 不可能以现在的形态存在：

- [Trainers' Legend G](https://github.com/MinamiChiwa/Trainers-Legend-G)
- [umamusume-localify-android](https://github.com/Kimjio/umamusume-localify-android)
- [umamusume-localify](https://github.com/GEEKiDoS/umamusume-localify)
- [Carotenify](https://github.com/KevinVG207/Uma-Carotenify)
- [umamusu-translate](https://github.com/noccu/umamusu-translate)
- [frida-il2cpp-bridge](https://github.com/vfsfitvnm/frida-il2cpp-bridge)

# 许可证
[GNU GPLv3](LICENSE)
