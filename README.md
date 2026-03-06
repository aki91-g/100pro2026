 [プロダクト概要](#japanese) | [Product Description](#english) | [開発者情報](#developer-jp)
<a name="japanese"></a>
# “今やるべきこと” が直感的に見えるタスク管理ツール
100プロ(2026春、第9期)にて始動。

## 課題
明日までにクリアしたいゲームのステージと、明後日までに書かなければいけないレポートがあるとき、より目につくべきタスクはどちらでしょうか？

従来のタスク管理ツールは、日程順に表示されます。

このデスクトップアプリはユーザーの意欲、期限、所要時間などを軸に視覚化することを目指します。

## 構想
- **カームな画面設計** 
- **多元的なタスク管理**
  - やる気(意欲)
  - 緊急性(期限)
  - 想定される所要時間
- **ローカルファースト**

## 将来的な構想
1. AIサジェスト機能
3. タスクに対する報酬(EXP)機能
4. ご褒美タスクのサジェスト
5. 外部アプリとの連携

## 技術スタック
|  |  |
| :--- | :--- |
| **Frontend** | Vue 3, Vite, TypeScript, Tailwind CSS |
| **Desktop** | Rust, Tauri |
| **Database** | SQLite, PostgreSQL |
| **Backend** | Hono, TypeScript|



<a name="english"></a>
# The Task Visualizer that understands your "Now."
Launchd in 100 programs, Spring 2026 (Generation 9)

## The Problem
When faced with a game task to finish by tomorrow and the final report due the day after, which one should be more prominent? 

Standard apps only look at dates. This desktop-app looks at how you feel and how long things take, ensuring the "right" task catches your eye at the right time.

## The Vision
- **Intuitive Visualization** 
- **Multi-Dimensional Sorting** based on:
  - Motivation
  - Urgency
  - Time Cost
- **Local-First, Cloud-Ready**

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
| **Database** | SQLite, PostgreSQL |
| **Backend** | Hono, TypeScript|




<a name="developer-jp"></a>
# 開発者情報 (For-Developers)
## システム構成
Tauri(Desktop) + Vue(Frontend) + Hono(Backend) をpnpm と Turborepo により統合したモノレポ構成のアプリ。

## MVP
- フロントエンド実装
- ローカルDB連携

※　Web実装と外部DB同期(PostgreSQL)については100プロ期間中の必須要件とはみなさない。

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










