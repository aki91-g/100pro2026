import { computed, ref } from 'vue';

export type ThemeMode = 'light' | 'dark';
export type LanguageCode = 'en' | 'ja';

const STORAGE_THEME_KEY = 'taskgraph.settings.theme';
const STORAGE_LANGUAGE_KEY = 'taskgraph.settings.language';

const theme = ref<ThemeMode>('light');
const language = ref<LanguageCode>('en');
const isInitialized = ref(false);

const messages = {
  en: {
    appTitle: 'TaskGraph',
    specialThanks: 'Special Thanks',
    helpTitle: 'How to use',
    guestMode: 'Guest Mode',
    guestLocalMode: 'Guest local mode',
    dbSyncing: 'Database syncing...',
    dbIdle: 'Database idle',
    theme: 'Theme',
    light: 'Light',
    dark: 'Dark',
    language: 'Language',
    english: 'English',
    japanese: 'Japanese',
    syncStatus: 'Sync Status',
    refresh: 'Refresh',
    maximizeGraph: 'Maximize Graph',
    exitFullscreen: 'Exit Fullscreen (Esc)',
    list: 'List',
    new: 'New',
    logout: 'Logout',
    logoutConfirm: 'Are you sure you want to logout?',
    welcome: 'Welcome',
    range1d: '1 day',
    range3d: '3 days',
    range1w: '1 week',
    range2w: '2 weeks',
    range1m: '1 month',
    axisDuration: 'Duration',
    axisMotivation: 'Motivation',
    axisStatus: 'Status',
    visualNone: 'None',
    controlWindow: 'Window',
    controlYAxis: 'Y-Axis',
    controlColor: 'Color',
    controlRadius: 'Radius',
    drawerCreateTask: 'Create Task',
    drawerDetails: 'Details',
    drawerEditTask: 'Edit Task',
    drawerBack: 'Back',
    drawerClose: 'Close',
    drawerTitle: 'Title',
    drawerTitleRequired: 'Title *',
    drawerTitlePlaceholder: 'Task name...',
    drawerDescription: 'Description',
    drawerDescriptionPlaceholder: 'Optional notes',
    drawerDue: 'Due',
    drawerDueRequired: 'Due *',
    drawerDurationMin: 'Duration (min)',
    drawerCancel: 'Cancel',
    drawerCreating: 'Creating...',
    drawerNoDescription: 'No description',
    drawerStatus: 'Status',
    drawerStatusAriaView: 'Task status in details view',
    drawerStatusAriaEdit: 'Task status in edit form',
    drawerMotivation: 'Motivation',
    drawerDuration: 'Duration',
    drawerMinuteUnit: 'min',
    drawerEdit: 'Edit',
    drawerSaveChanges: 'Save Changes',
    drawerArchive: 'Archive',
    drawerDelete: 'Delete',
    drawerDeleteConfirm: 'Are you sure you want to delete this task?',
    statusBacklog: 'Backlog',
    statusTodo: 'Todo',
    statusDoing: 'Doing',
    statusDone: 'Done',
    groupedTasks: 'Grouped Tasks',
    groupedTasksCount: 'tasks grouped',
    statusPrefix: 'Status',
    past: 'Past',
    future: 'Future',
    thanksClose: 'Close',
    thanksDevelopedBy: 'Developed by',
    thanksGithubProfile: 'GitHub Profile',
    thanksIntro: 'TaskGraph was created during {program}.',
    thanksOfficialSite: '100 program Official Site',
    thanksSectionTitle: 'Special Thanks to',
    thanksFeedback: 'Feedback / フィードバック協力',
    thanksTeam: 'Team Members / チームメンバー',
    thanksCopyright: 'TaskGraph Project',
  },
  ja: {
    appTitle: 'TaskGraph',
    specialThanks: '謝意',
    helpTitle: '使い方',
    guestMode: 'ゲストモード',
    guestLocalMode: 'ゲストローカルモード',
    dbSyncing: 'データベース同期中...',
    dbIdle: 'データベース待機中',
    theme: 'テーマ',
    light: 'ライト',
    dark: 'ダーク',
    language: '言語',
    english: '英語',
    japanese: '日本語',
    syncStatus: '同期状態',
    refresh: '更新',
    maximizeGraph: 'グラフを最大化',
    exitFullscreen: '全画面を終了 (Esc)',
    list: '一覧',
    new: '新規',
    logout: 'ログアウト',
    logoutConfirm: 'ログアウトしますか？',
    welcome: 'ようこそ',
    range1d: '1日',
    range3d: '3日',
    range1w: '1週間',
    range2w: '2週間',
    range1m: '1か月',
    axisDuration: '時間',
    axisMotivation: 'モチベーション',
    axisStatus: 'ステータス',
    visualNone: 'なし',
    controlWindow: '表示範囲',
    controlYAxis: 'Y軸',
    controlColor: '色',
    controlRadius: '半径',
    drawerCreateTask: 'タスク作成',
    drawerDetails: '詳細',
    drawerEditTask: 'タスク編集',
    drawerBack: '戻る',
    drawerClose: '閉じる',
    drawerTitle: 'タイトル',
    drawerTitleRequired: 'タイトル *',
    drawerTitlePlaceholder: 'タスク名...',
    drawerDescription: '説明',
    drawerDescriptionPlaceholder: '任意メモ',
    drawerDue: '期限',
    drawerDueRequired: '期限 *',
    drawerDurationMin: '所要時間 (分)',
    drawerCancel: 'キャンセル',
    drawerCreating: '作成中...',
    drawerNoDescription: '説明なし',
    drawerStatus: 'ステータス',
    drawerStatusAriaView: '詳細画面のタスクステータス',
    drawerStatusAriaEdit: '編集画面のタスクステータス',
    drawerMotivation: 'モチベーション',
    drawerDuration: '所要時間',
    drawerMinuteUnit: '分',
    drawerEdit: '編集',
    drawerSaveChanges: '変更を保存',
    drawerArchive: 'アーカイブ',
    drawerDelete: '削除',
    drawerDeleteConfirm: 'このタスクを削除しますか？',
    statusBacklog: 'バックログ',
    statusTodo: '未着手',
    statusDoing: '進行中',
    statusDone: '完了',
    groupedTasks: 'グループ化タスク',
    groupedTasksCount: '件のタスク',
    statusPrefix: 'ステータス',
    past: '過去',
    future: '未来',
    thanksClose: '閉じる',
    thanksDevelopedBy: '開発者',
    thanksGithubProfile: 'GitHubプロフィール',
    thanksIntro: 'TaskGraphは{program}で制作されました。',
    thanksOfficialSite: '100 program 公式サイト',
    thanksSectionTitle: '謝意',
    thanksFeedback: 'Feedback / フィードバック協力',
    thanksTeam: 'Team Members / チームメンバー',
    thanksCopyright: 'TaskGraph プロジェクト',
  },
} as const;

type TranslationKey = keyof (typeof messages)['en'];
type TranslationParams = Record<string, string | number>;

function isThemeMode(value: string | null): value is ThemeMode {
  return value === 'light' || value === 'dark';
}

function isLanguageCode(value: string | null): value is LanguageCode {
  return value === 'en' || value === 'ja';
}

function applyThemeClass(nextTheme: ThemeMode): void {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('dark', nextTheme === 'dark');
}

function persistSettings(): void {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(STORAGE_THEME_KEY, theme.value);
  window.localStorage.setItem(STORAGE_LANGUAGE_KEY, language.value);
}

function initializeSettings(): void {
  if (isInitialized.value) return;

  if (typeof window !== 'undefined') {
    const savedTheme = window.localStorage.getItem(STORAGE_THEME_KEY);
    const savedLanguage = window.localStorage.getItem(STORAGE_LANGUAGE_KEY);

    if (isThemeMode(savedTheme)) {
      theme.value = savedTheme;
    } else if (window.matchMedia?.('(prefers-color-scheme: dark)').matches) {
      theme.value = 'dark';
    }
    if (isLanguageCode(savedLanguage)) {
      language.value = savedLanguage;
    }
  }

  applyThemeClass(theme.value);
  persistSettings();
  isInitialized.value = true;
}

export function useSettings() {
  function setTheme(nextTheme: ThemeMode): void {
    theme.value = nextTheme;
    applyThemeClass(nextTheme);
    persistSettings();
  }

  function toggleTheme(): void {
    setTheme(theme.value === 'light' ? 'dark' : 'light');
  }

  function setLanguage(nextLanguage: LanguageCode): void {
    language.value = nextLanguage;
    persistSettings();
  }

  function toggleLanguage(): void {
    setLanguage(language.value === 'en' ? 'ja' : 'en');
  }

  function t(key: TranslationKey, params?: TranslationParams): string {
    const base = String(messages[language.value][key] ?? messages.en[key] ?? key);
    if (!params) return base;

    return Object.entries(params).reduce((accumulator, [paramKey, value]) => {
      return accumulator.replaceAll(`{${paramKey}}`, String(value));
    }, base);
  }

  return {
    theme: computed(() => theme.value),
    language: computed(() => language.value),
    initializeSettings,
    setTheme,
    toggleTheme,
    setLanguage,
    toggleLanguage,
    t,
  };
}
