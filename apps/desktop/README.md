## LOCAL-FIRST SYNC
LOCAL CREATION:
Item created locally → sync_status = 'local_only'
    ↓
sync_local_to_remote() → Included (status != 'synced')
    ↓ (push to Postgres)
Remote marked 'synced' → Local marked 'synced'
    ↓
Next cycle: sync_local_to_remote() → Skipped

PULL FROM REMOTE:
sync_items() → All Postgres items
    ↓
Check if exists locally
    ↓
Create/upsert in local DB → Mark 'synced' immediately
    ↓ (prevents re-upload)
If exists locally: Refreshed
If created remotely: log as newly pulled item.

MODIFIED ITEM:
User edits local item → sync_status marked 'modified'
    ↓
sync_local_to_remote() → Included (status != 'synced')
    ↓ (push changes to Postgres)
Both sides marked 'synced'

FAILURE RECOVERY:
Failed push Status update → Status unchanged
    ↓
Next sync attempt → Retried immediately


## Directory
以下はGitHub Actionsによって、pushごとに最新のツリーに入れ替わります。
タグを編集しないでください。

[TREE-START]
```text
apps/desktop
├── src-tauri
│   ├── capabilities
│   │   └── default.json
│   ├── gen
│   │   └── schemas
│   ├── icons
│   │   ├── 128x128@2x.png
│   │   ├── 128x128.png
│   │   ├── 32x32.png
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   ├── icon.png
│   │   ├── Square107x107Logo.png
│   │   ├── Square142x142Logo.png
│   │   ├── Square150x150Logo.png
│   │   ├── Square284x284Logo.png
│   │   ├── Square30x30Logo.png
│   │   ├── Square310x310Logo.png
│   │   ├── Square44x44Logo.png
│   │   ├── Square71x71Logo.png
│   │   ├── Square89x89Logo.png
│   │   └── StoreLogo.png
│   ├── migrations
│   │   ├── postgres
│   │   └── sqlite
│   ├── permissions
│   ├── src
│   │   ├── commands
│   │   ├── database
│   │   ├── models
│   │   ├── repositories
│   │   ├── services
│   │   ├── utils
│   │   ├── error.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── state.rs
│   ├── tests
│   │   └── db_test.rs
│   ├── build.rs
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── tauri.conf.json
├── package.json
├── README.md
├── tsconfig.json
├── tsconfig.node.json
└── vite.config.ts
```
[TREE-END]









