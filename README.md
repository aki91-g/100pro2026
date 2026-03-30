 [プロダクト概要](#japanese) | [Product Description](#english) | [開発者情報](#developer-jp)
# DEMO


https://github.com/user-attachments/assets/8f75b696-75cc-4463-abd0-d3e2c7e6948c



<a name="japanese"></a>
# “今やるべき” を可視化するタスク管理ツール
100プロ(2026春、第9期)にて始動。5週間の開発期間で実装。

## 課題：脳のバイアスによる「優先順位の誤認」

明日までのゲームイベントと、明後日〆切のレポート。本来優先すべきは後者ですが、私たちの脳は「手軽な報酬」や「新しさ」にバイアスがかかり、重要度の低いタスクを優先してしまう性質があります。

従来のリスト形式のツールでは、文字情報（タイトル）自体が誘惑や危機感を引き起こし、冷静な判断を妨げることがありました。

## 解決策：無機質な点による「多元的な可視化」
TaskGraphは、タスクをタイトルではなく **「無機質な点」** としてグラフ上にプロットします。

感情を排除し、データに基づいた配置を行うことで、真に今やるべきことに向き合う体験を提供します。

## 特徴
**カームな画面設計**
- モダンなUI/UX
- ダークモード
- 二言語対応 (EN/JP)
**多元的な軸による可視化**
  - やる気(意欲)
  - 緊急性(期限)
  - 想定される所要時間
 **ローカルファースト & マルチプラットフォーム**
- デスクトップ版（Tauri/Rust）はオフラインでも利用可能なローカルDB（SQLite）駆動
- Web版とのシームレスなデータ同期

## 将来的な構想
1. AIサジェスト機能
2. タスクに対する報酬(EXP)機能
3. 外部アプリとの連携

## 技術スタック
|  |  |
| :--- | :--- |
| **Frontend** | Vue 3, Vite, TypeScript, Tailwind CSS |
| **Desktop** | Rust, Tauri |
| **Database** | SQLite, PostgreSQL, Supabase (Auth/DB)|
| **Backend** | Hono, TypeScript|
| **Deploy**  | Render |



<a name="english"></a>
# The Task Visualizer that truely shows your "Now."
Launchd in 100 programs, Spring 2026 (Generation 9)

## The Problem: Misprioritization due to Cognitive Bias
Faced with a game event due tomorrow and a report due the day after, our brains often prioritize the "easy reward" of the game. 

Traditional list-based apps can exacerbate this, as text-heavy titles often trigger emotional distractions.

## The Solution: Multi-dimensional Visualization with Neutral Data points
TaskGraph plots tasks as "points" on a graph rather than text entries. 

By abstracting tasks into data points, it eliminates emotional noise and helps users face what truly matters.

## Key Features
- **Intuitive Visualization**
- Minimalist design
- Light/Dark Mode
- Bilingual support (EN/JP)
- **Multi-Dimensional Sorting** based on:
  - Motivation
  - Due
  - Duration
- **Local-First, Cloud-Ready**
- Desktop app (Tauri/Rust) runs offline with a local SQLite database.
- Synchronizes with the Web version for cross-platform accessibility.

## Future Roadmap
1. AI suggetion
3. Gamification & EXP
4. The "Reward" Loop
5. Ecosystem Sync

## Tech Stack
|  |  |
| :--- | :--- |
| **Frontend** | Vue 3, Vite, TypeScript, Tailwind CSS |
| **Desktop** | Rust, Tauri |
| **Database** | SQLite, PostgreSQL, Supabase (Auth/DB)|
| **Backend** | Hono, TypeScript|
| **Deploy**  | Render |



<a name="developer-jp"></a>
# 開発者情報 (For-Developers)
## システム構成
Tauri(Desktop) + Vue(Frontend) + Hono(Backend) をpnpm と Turborepo により統合したモノレポ構成のアプリ。

## 開発フロー
### 開発環境 (Dev Container)
1. VS CodeのCommand Paletteを開く。
    | OS | ショートカットキー |
    | :--- | :--- |
    | Windows | Ctrl + Shift + P |
    | Mac| Command (⌘) + Shift + P|

2. 「Dev Containers: Reopen in Container」を選択。
    
    自動的にNode.js / Rust / pnpm / 各種ライブラリがセットアップされる。
3. Dev Containerに入っていることを確認する。
    
    VS Codeウィンドウの左下のバーに「Dev Container: 100pro2026-devcontainer」と表示されていればOK。
    
    または、ターミナルのパスが `/devcontainer` になっていればOK。


### 動作確認
Dev Containerに入っていることを確認。

プロジェクトルート(/devcontainer)で以下を実行。
```bash
pnpm install
pnpm dev
```
1. Frontend (Vue): localhost:5173 で起動。
2. Backend (Hono): localhost:3000 で API サーバーが起動。
3. Desktop (Tauri): Rust のコンパイル後、デスクトップウィンドウが自動で開きます。

    初回はRustのコンパイルに時間がかかります。


### 開発サイクル
1. ブランチを切る。
```bash
# リモートブランチに追いつく
git pull origin main

# mainブランチに移動
git checkout main

# 新しいブランチを作成して移動
git checkout -b front/[TASKNAME]
```
2. コードを書き、ローカルで動作確認する。
3. gitで作業内容を保存する。
```bash
# 変更内容を一時保存
git add .

# addされている内容を確認
git status

# メッセージを付けて変更内容をローカルブランチに保存
git commit -m "[メッセージ]"
 
# ローカルブランチに保存された内容をリモートブランチに保存
git push origin front/[TASKNAME]
```

4. GitHub 上で Pull Request (PR) を作成
```text
1. GitHubのリポジトリ上で「Compare & pull request」を選択。

2. Base: main ← Compare: front/task-name になっているか確認。

3. やったことを記載する。可能であれば、動作確認の手順も記載する。

4. 「Create pull request」を選択。
```

5. レビューを受ける。

6. 承認（Approve）を得たら「Merge pull request」を選択する

### gitトラブルシューティング
```bash
# 自分がどのブランチにいるか確認したいとき
git branch

# 一時保存(add)をキャンセルしたいとき
git reset

# 保存(commit)をキャンセルしたいとき
git reset --soft HEAD^

### 間違えてmainブランチで作業してしまった場合 ###

# まだコミットしていないとき
git checkout -b front/[TASKNAME]

# すでにコミットしてしまったとき
git reset --soft HEAD^
git checkout -b front/TASKNAME
```

## CODEOWNERS
1. mainブランチへの直接pushは禁止

2. 全ての変更はプルリク必須

3. Rust部分への変更はaki91-gの承認が必要

## CI (GitHub Actions)
既定の変更を含む場合、mainブランチへのプルリクが作成されると、`.github/workflows/ci.yml`が作動し以下のチェックが行われます。
1. Environment Setup: Node.js 20 / pnpm の環境構築。

2. Rust Toolchain: Stable Rust のインストール。

3. System Dependencies: Tauri ビルドに必須な Linux ライブラリ（GTK, WebKit2GTK 等）の導入。

4. Monorepo Build: pnpm build を実行。Turbo により Frontend → Backend → Desktop の順でビルドと型チェックが並列実行されます。

## 通信
1. Web通信: Vue → Fastify。Dockerネットワーク経由のHTTP通信。

2. Tauri IPC: WebView → Rust。PC固有機能への命令。
    | ホスト | コード |
    | :--- | :--- |
    | Rust側 | `#[tauri::command]`を追記すると、フロント側から呼び出し可能 |
    | TS側 | `invoke('関数名')` でRust側の処理を非同期呼び出し |


## Directory
以下はGitHub Actionsによって、pushごとに最新のツリーに入れ替わります。
タグを編集しないでください。

[TREE-START]
```text
.
├── apps
│   ├── backend
│   │   ├── src
│   │   ├── package.json
│   │   ├── README.md
│   │   └── tsconfig.json
│   ├── desktop
│   │   ├── src-tauri
│   │   ├── package.json
│   │   ├── README.md
│   │   ├── tsconfig.json
│   │   ├── tsconfig.node.json
│   │   └── vite.config.ts
│   └── frontend
│       ├── public
│       ├── src
│       ├── ARCHITECTURE.md
│       ├── index.html
│       ├── package.json
│       ├── README.md
│       ├── tsconfig.app.json
│       ├── tsconfig.json
│       ├── tsconfig.node.json
│       └── vite.config.ts
├── docker
│   └── ci.Dockerfile
├── scripts
│   └── update-trees.sh
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── README.md
└── turbo.json
```
[TREE-END]











