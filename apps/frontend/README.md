# Vue 3 + TypeScript + Vite

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

Learn more about the recommended Project Setup and IDE Support in the [Vue Docs TypeScript Guide](https://vuejs.org/guide/typescript/overview.html#project-setup).

## Recent UI Updates

- ScatterPlot theme sync now repaints after `nextTick()` on theme changes to avoid dark/light transition lag.
- Canvas background and grid lines now map directly from `useSettings.theme` for stable first-frame rendering.
- Task list status pills use shared CSS variables for instant light/dark readability.
- Special Thanks modal labels are now routed through the i18n `t()` helper.
- Brand badge background remains controlled by `--badge-bg` (kept white in both themes).

## Directory
以下はGitHub Actionsによって、pushごとに最新のツリーに入れ替わります。
タグを編集しないでください。

[TREE-START]
```text
apps/frontend
├── public
│   └── vite.svg
├── src
│   ├── api
│   │   ├── authRepository.ts
│   │   ├── config.ts
│   │   ├── honoClient.ts
│   │   └── itemRepository.ts
│   ├── assets
│   │   └── vue.svg
│   ├── components
│   │   ├── ScatterPlot.vue
│   │   ├── SyncStatusBadge.vue
│   │   ├── TaskDrawer.vue
│   │   └── TaskList.vue
│   ├── composables
│   │   ├── useAuth.ts
│   │   ├── useGraph.ts
│   │   ├── useItems.ts
│   │   └── useSyncStatus.ts
│   ├── layouts
│   ├── stores
│   │   └── user.ts
│   ├── types
│   │   ├── graph.ts
│   │   └── item.ts
│   ├── views
│   │   ├── LoginView.vue
│   │   └── MainDashboard.vue
│   ├── App.vue
│   ├── main.ts
│   ├── style.css
│   └── vite-env.d.ts
├── ARCHITECTURE.md
├── index.html
├── package.json
├── README.md
├── tsconfig.app.json
├── tsconfig.json
├── tsconfig.node.json
└── vite.config.ts
```
[TREE-END]











