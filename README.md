## システム構成
Tauri(Desktop) + Vue(Frontend) + Hono(Backend) をpnpm と Turborepo により統合したモノレポ構成のアプリ。

## 技術スタック
| 分類 | 技術 |
| :--- | :--- |
| **Frontend** | Vue 3, Vite, TypeScript, Tailwind CSS |
| **Desktop** | Rust, Tauri |
| **Database** | SQLite |
| (**Backend**) | Hono, TypeScript|

backendとdatabaseについては必要に応じて拡張予定。


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
プルリクが作成されると、`.github/workflows/ci.yml`が作動し以下のチェックが行われます。
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


## ディレクトリ構成
apps以下の詳細は各ドキュメントで確認してください。

```text
.
├── .devcontainer/
│   ├── .gitattributes
│   ├── devcontainer.json
│   └── Dockerfile
├── .github/
│   ├── workflows/
│   │   └── ci.yml
│   └── CODEOWNERS
├── apps/
│   ├── backend/
│   ├── desktop/
│   └── frontend/
├── .gitignore
├── LICENSE
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── README.md
└── turbo.json
```