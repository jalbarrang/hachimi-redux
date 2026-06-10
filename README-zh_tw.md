<img align="left" width="80" height="80" src="apps/hachimi/assets/icon.png">

# HachimiRedux

[English](README.md) | [简体中文](README-zh_cn.md) | 繁體中文 | [Español (España)](README-es_es.md) | [Español (Latinoamérica)](README-es_419.md) | [Français](README-fr_fr.md) | [Português (Brasil)](README-pt_br.md) | [Português (Portugal)](README-pt_pt.md)

針對 UM:PD 的遊戲增強與翻譯 Mod。HachimiRedux 是 Hachimi 的一個分支，內建了遊戲內的養成追蹤插件，並重寫了原生插件 SDK。

<img height="400" src="apps/hachimi/assets/screenshot-2.png">

# ⚠️ 請不要連結到本儲存庫或 Hachimi 的網站
我們理解你想幫助大家安裝 Hachimi 並獲得更好的遊戲體驗。然而，本專案本質上違反了遊戲的服務條款，遊戲開發者一旦得知它的存在，幾乎肯定會希望將其剷除。

在你自行管理的聊天服務中以及透過私訊分享是可以的，但我們懇請你不要在面向公眾的網站上分享本專案的連結，也不要分享任何相關工具的連結。

或者你也可以照樣分享，把它給那為數不多的 Hachimi 使用者毀掉。這取決於你。

### 如果你無論如何都要分享
你想怎麼做都行，但我們懇請你盡量將遊戲標註為「UM:PD」或「The Honse Game」，而不是遊戲的真實名稱，以避免被搜尋引擎抓取。

# ⚠️ 與上游 Hachimi 插件不相容
本分支自帶其專屬的原生插件 API（host API v9）。**為上游 Hachimi 建置的插件無法在 HachimiRedux 上使用**，而這裡發佈的養成追蹤插件也無法在上游 Hachimi 上載入。請優先使用從本儲存庫建置的 DLL，並將它們搭配使用。混用不同的建置可能導致載入失敗或遊戲當機。

## 舊版插件相容（選用，受限）
無資訊清單、採用舊版 ABI 的插件（例如上游 Hachimi 的資料匯出器）可以透過一條**選用的相容路徑**載入。除了 `load_libraries` 之外，還需在 `config.json` 中的 `legacy_libraries` 白名單裡列出該 DLL：

```json
{
  "windows": {
    "load_libraries": ["some_legacy_plugin.dll"],
    "legacy_libraries": ["some_legacy_plugin.dll"]
  }
}
```

舊版插件只需匯出 `hachimi_init`；宿主會跳過其通常的資訊清單/ABI 檢查，並基於信任載入它。此支援是**受限且不受官方支援的**：

- 該插件必須**僅依賴宿主 API 的穩定 vtable 前綴**。超出此範圍的任何用法都屬於未定義行為，可能導致遊戲當機。
- 宿主**無法驗證、追蹤或卸載**舊版插件及其 IL2CPP 掛鉤。該 DLL 會在整個處理程序生命週期內保持對應。
- 每當一個插件透過此路徑載入時，都會記錄一條警告。
- `legacy_libraries` 中的項目也必須出現在 `load_libraries` 中。

如有疑問，請針對本儲存庫（host API v9）重新編譯插件，而不要依賴舊版路徑。

# 特色
- **高品質翻譯：** Hachimi 提供了先進的翻譯功能，讓譯文更加自然（複數形式、序數詞等），並避免給 UI 帶來錯亂。它還支援翻譯遊戲中的大多數元件，無需手動修補替換資源！

    支援的元件：
    - UI 文字
    - master.mdb（技能名稱、技能描述等）
    - 比賽劇情
    - 主線劇情/主頁對話
    - 歌詞
    - 紋理替換
    - 精靈圖集替換

    此外，Hachimi 並非只為某一種語言提供翻譯功能；它在設計上可針對任何語言完全自訂。

- **輕鬆上手：** 隨插即用。所有設定都在遊戲內完成，無需任何外部程式。
- **翻譯自動更新：** 內建的翻譯更新器讓你照常遊玩的同時進行更新，完成後在遊戲內重新載入，無需重新啟動！
- **內建圖形介面：** 自帶設定編輯器，讓你無需離開遊戲即可修改設定！
- **圖形設定：** 你可以調整遊戲的圖形設定，以充分發揮裝置效能，例如解鎖 FPS 和解析度縮放。
- **僅限 Windows：** 專為遊戲的 Windows（Steam）版本打造。**HachimiRedux 出於自身選擇不支援 Android** —— 它只專注於 Windows 用戶端，沒有計畫新增或維護 Android 版本。

# 安裝

HachimiRedux 是核心 Mod（以 `cri_mana_vpx.dll` 的形式載入）；**Training Tracker（養成追蹤器）** 是由核心 Mod 載入的選用插件 DLL。兩者都從本儲存庫建置，並且必須來自同一次建置。

遊戲目錄即 Steam 安裝資料夾，例如
`C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby`。

## 從原始碼建置

本儲存庫是一個 Cargo workspace。在儲存庫根目錄下：

```sh
# 核心 Mod
cargo build --release -p hachimi                    # -> target/release/hachimi.dll
# Training Tracker 插件
cargo build --release -p hachimi-training-tracker   # -> target/release/hachimi_training_tracker.dll
```

## 安裝 HachimiRedux（核心）

遊戲透過算繪器 DLL `cri_mana_vpx.dll` 載入該 Mod。

1. 在遊戲目錄中，將原始的 `cri_mana_vpx.dll` 備份為 `cri_mana_vpx.dll.backup`（只做一次 —— 之後切勿覆寫該備份）。
2. 將 `target/release/hachimi.dll` 複製到遊戲目錄，並重新命名為 `cri_mana_vpx.dll`。
3. 啟動遊戲。按下選單鍵 —— 預設是**右方向鍵** —— 開啟遊戲內 UI。啟動畫面會顯示目前的按鍵，你也可以在遊戲內圖形介面中重新繫結它。

Mod 設定儲存在遊戲資料目錄中的 `config.json` 內，該目錄是**遊戲目錄下的 `hachimi` 子資料夾**（例如 `…\UmamusumePrettyDerby\hachimi\config.json`）。它會由安裝器/首次啟動時自動建立；其餘所有內容都透過遊戲內圖形介面進行設定。

## 安裝 Training Tracker 插件

插件是核心 Mod 在啟動時從遊戲目錄根部載入的原生 DLL。

1. 先安裝 HachimiRedux 核心（見上文）。
2. 將 `target/release/hachimi_training_tracker.dll` 複製到遊戲目錄根部（與 `cri_mana_vpx.dll` 同一資料夾）。注意：插件 DLL 放在遊戲**根目錄**，而 `config.json` 位於 `hachimi` 子資料夾中。
3. 在 `config.json`（`<game_dir>\hachimi\config.json`）的 `load_libraries` 清單中加入該 DLL：

   ```json
   {
     "windows": {
       "load_libraries": ["hachimi_training_tracker.dll"]
     }
   }
   ```
4. 啟動遊戲。追蹤器會作為 Plugins 分頁中的一個頁面出現，同時也作為一個浮動的覆蓋面板。關於插件的運作方式，請參閱 [docs/plugin-sdk.md](docs/plugin-sdk.md)。

## 自動部署（Windows，從原始碼）

在儲存庫根目錄下，輔助指令碼會建置並將兩個 DLL 複製到遊戲目錄：

```powershell
.\scripts\deploy-windows.ps1 -Build
```

如果遊戲不在預設的 Steam 路徑，可覆寫遊戲資料夾：

```powershell
$env:HACHIMI_GAME_DIR = "D:\path\to\UmamusumePrettyDerby"
.\scripts\deploy-windows.ps1 -Build
```

該指令碼會將 `hachimi.dll` → `cri_mana_vpx.dll` 以及 training tracker DLL 複製到遊戲目錄，且絕不會修改 `cri_mana_vpx.dll.backup`。

# 疑難排解

## 遊戲啟動時當機／行為異常

到目前為止最常見的原因是在遊戲資料夾中**堆疊了多個遊戲 Mod 或 DLL 注入器**。每一個都會掛鉤遊戲的算繪/執行階段，互相爭搶。HachimiRedux 會在遊戲內（一條通知 + `hachimi.log`）對此發出警告，安裝器也會在安裝前發出警告，但你必須自行移除其他的：

- **只**保留 HachimiRedux：`cri_mana_vpx.dll` 以及任何由 HachimiRedux 建置的插件（例如 `hachimi_training_tracker.dll`）。
- 從遊戲資料夾中移除其他覆蓋層/注入器，例如不該出現的代理載入器 DLL（`version.dll`、`winhttp.dll`、`dxgi.dll`、`d3d11.dll`、`dinput8.dll` 等）以及具名覆蓋層（`horseACT.dll`、`heaven_overlay.dll` 等）。
- **只有由 HachimiRedux 建置的插件**才應放入 `load_libraries`。不要在其中加入第三方覆蓋層 —— 它們不是 HachimiRedux 插件，會被拒絕（並附帶遊戲內提示），或可能導致遊戲當機。

## 各檔案位置

- `cri_mana_vpx.dll` 和插件 DLL：遊戲**根**目錄。
- `config.json` 及其他 Mod 資料：遊戲目錄的 **`hachimi` 子資料夾**（`<game_dir>\hachimi\config.json`）。
- Mod 紀錄：遊戲根目錄下的 `hachimi.log`（在 `config.json` 中啟用 `enable_file_logging`）。
- 遊戲紀錄：`%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log`。

## 收集診斷資訊

- 遊戲內：開啟選單（預設右方向鍵）→ **Config** → **Save diagnostics report**。這會在遊戲資料夾中寫入 `hachimi_diagnostics.txt`。
- 安裝器：執行 `installer collect-logs`，將 `config.json`、`hachimi.log` 以及一份衝突報告收集到 `%TEMP%\hachimi_diagnostics`。

# 特別鳴謝

HachimiRedux 是一個建立在以下專案工作之上的分支：

- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi) —— 本專案所基於的原始專案。如果你對原始專案有興趣，歡迎加入[它的 Discord 伺服器](https://discord.gg/YjBgmuqqYr)。
- [Hachimi Edge](https://github.com/kairusds/Hachimi-Edge) —— HachimiRedux 所延續的、專注於 Windows/Steam 的分支。

這些專案又反過來成為了 Hachimi 開發的基礎；沒有它們，Hachimi 不可能以現在的形態存在：

- [Trainers' Legend G](https://github.com/MinamiChiwa/Trainers-Legend-G)
- [umamusume-localify-android](https://github.com/Kimjio/umamusume-localify-android)
- [umamusume-localify](https://github.com/GEEKiDoS/umamusume-localify)
- [Carotenify](https://github.com/KevinVG207/Uma-Carotenify)
- [umamusu-translate](https://github.com/noccu/umamusu-translate)
- [frida-il2cpp-bridge](https://github.com/vfsfitvnm/frida-il2cpp-bridge)

# 授權
[GNU GPLv3](LICENSE)
