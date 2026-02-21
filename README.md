## システム構成
Tauri(Desktop) + Vue(Frontend) + Hono(Backend) をpnpm と Turborepo により統合したモノレポ構成のアプリ。

frontend、backendはDocker上で動作します。


## 技術スタック
| 分類 | 技術 |
| :--- | :--- |
| **Frontend** | Vue 3, Vite, TypeScript, Tailwind CSS |
| **Desktop** | Rust, Tauri |
| **Database** | SQLite |
| (**Backend**) | Hono, TypeScript (Docker) |

backendとdatabaseについては必要に応じて拡張予定。


## 開発フロー
### 開発環境 (Dev Container)
1. VS CodeのCommand Paletteを開く。
    - Windows: Ctrl + Shift + P
    - Mac: Command (⌘) + Shift + P
2. 「Dev Containers: Reopen in Container」を選択。
    - 自動的にNode.js / Rust / pnpm / 各種ライブラリがセットアップされる。
3. Dev Containerに入っていることを確認する。
    - VS Codeウィンドウの左下のバーに「Dev Container: 100pro2026-devcontainer」と表示されていればOK。
    - または: 新しくターミナルを開き、パスが `/devcontainer` になっていればOK。


### 動作確認
1. プロジェクトルート(/devcontainer)で
    `pnpm install`
    を実行
3. プロジェクトルート(/devcontainer)で
    `pnpm dev`
    を実行
    - frontendとbackendがDocker上で起動し、localhostで参照可能になる。
    - Rustのコンパイルが行われ、Desktopウィンドウが開く。(少し時間がかかります)

### 開発サイクル
1. `git pull origin main`でリモートブランチに追いつく。
2. `git checkout main`でmainブランチに移動。
3. `git checkout -b front/TASKNAME`で新しいブランチを作成して移動。
4. コードを書き、ローカルで動作確認。
5. `git add .` で変更内容を一時保存。
6. `git status`または VS CodeのSource Controlでaddされている内容を確認。
    - 内容が間違っている場合は、`git reset`で一時保存の内容をキャンセル。
7. `git commit -m "内容を簡潔に"`でメッセージを付けて変更内容をローカルブランチに保存。
    - 内容が間違っている場合は、`git reset --soft HEAD^`で保存の内容をキャンセルし、`git add .`からやり直す。
8. `git push origin front/TASKNAME`でリモートブランチに保存。
9. GitHub 上で Pull Request (PR) を作成。
    1. リポジトリのページで「Compare & pull request」を選択
    2. Base: main ← Compare: front/task-name になっているか確認。
    3. やったことを記載する。可能であれば、動作確認の手順も記載する。
    4. 「Create pull request」を選択。
10. レビューを受ける。
11. 承認（Approve）を得たら「Merge pull request」を選択して完了。
12. `git pull origin main`でリモートブランチに追いつく。

### 上手くいかない場合:
- backendに繋がらない/tauriがlocalhost警告を出す

    `docker compose ps`でfrontendとbackendの2つのDockerコンテナが起動しているか確認してください。起動していない場合は、`docker compose up -d`で起動してください。

- 画面が真っ白/接続拒否される

    `pnpm dev`での起動直後はViteの準備やRustのコンパイルに時間がかかります。ターミナル上で実行が完了しているか確認してください。完了していない場合は、しばらく待機してください。

- 必要なモジュールがインストールされていない

    プロジェクトルート(/devcontainer)で
    `pnpm install`
    を実行してください。

- `pnpm install`でエラーが出る
    1. sh: pnpm: command not foundの場合

        Dev Containerに入っていないか、起動に失敗しています。Dev Containerに入っていることを確認してください。入っていれば、Command Paletteから「Dev Containers: Rebuild Container」を選択してください。(上記参照)

    2. ERR_PNPM_LOCKFILE_CONFIG_MISMATCHの場合

        `pnpm-lock.yaml`が現在の環境と衝突しているため、ロックファイルをいったん無視して、インストールし直すため、
        `pnpm install --no-frozen-lockfile`
        を実行してください

    3. EACCES: permission deniedの場合

        `rm -rf node_modules **/node_modules`
        を実行し、node_modules を作り直してから、もう一度`pnpm install`を実行してください。

- 自分がどのブランチにいるか分からなくなった。

    `git branch`で確認できます。

- 間違えてmainブランチで作業してしまった。
    1. まだコミットしていない場合

        `git checkout -b front/TASKNAME`で作業内容を保持したまま新しいブランチを作成して移動し、そのまま作業してください。

    2. コミットしてしまった場合

        1. `git reset --soft HEAD^`で変更内容を保持したまま、1つ前の状態に戻してください。
        2. `git checkout -b front/TASKNAME`で作業内容を保持したまま新しいブランチを作成して移動し、そのまま作業してください。



## 通信
1. Web通信: Vue → Fastify。Dockerネットワーク経由のHTTP通信。

2. Tauri IPC: WebView → Rust。PC固有機能への命令。
    - Rust側: `#[tauri::command]`を追記すると、フロント側から呼び出し可能。
    - TS側: `invoke('関数名')` でRust側の処理を非同期呼び出し。

## CI/自動化
**GitHub Actions**: プルリクが作成されると、.github/workflows/ci.ymlが作動し以下のチェックが行われます。
1. Setup Node/pnpm: Node.js 20/pnpm の環境構築
2. Rust Toolchain: Stable Rust のインストール
3. System Libs: Tauriビルドに必須なライブラリの導入(Ubuntu環境)
4. cargo check: Rust側のコンパイルエラーの確認

**mainブランチ保護/CODEOWNERS**:
1. mainブランチへの直接pushは禁止
2. 全ての変更はプルリク必須
3. Rust部分への変更はaki91-gの承認が必要


## ディレクトリ構成
apps以下の詳細は各ドキュメントで確認してください。

```text
.
├── .devcontainer/           # Dev Container設定
├── .github/                 # GitHub 固有設定
│   ├── workflows/           # GitHub Actions(CI)設定
│   │   └── ci.yml           # PR時の自動ビルド・チェック
│   └── CODEOWNERS           # フォルダごとの責任者定義
├── apps/
│   ├── backend/             # 【Backend】 Hono (TypeScript)
│   ├── desktop/             # 【Desktop】 Tauri (Rust)
│   └── frontend/            # 【Frontend】 Vue 3 (Vite + TS)
├── .gitattributes           # 改行コード(LF)固定、バイナリ管理
├── .gitignore               # git除外設定
├── docker-compose.yml       # 開発環境全体のオーケストレーション
├── LICENSE                  # LICENSE定義(MIT License)
├── package.json             # モノレポ全体の依存管理 (pnpm workspaces)
├── pnpm-lock.yaml           # pnpm パッケージのバージョンロック
├── pnpm-workspace.yaml      # pnpm モノレポの定義
└── turbo.json               # Turborepo 設定（ビルド・実行の最適化）
```