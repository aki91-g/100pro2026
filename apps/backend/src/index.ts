import { serve } from '@hono/node-server';
import type { Context, MiddlewareHandler } from 'hono';
import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { createClient } from '@supabase/supabase-js';

const SUPABASE_URL = process.env['SUPABASE_URL'] as string;
const SUPABASE_ANON_KEY = process.env['SUPABASE_ANON_KEY'] as string;
const port = process.env.PORT || 10000;

if (!SUPABASE_URL || !SUPABASE_ANON_KEY) {
  throw new Error('SUPABASE_URL and SUPABASE_ANON_KEY must be set');
}

type TaskStatus = 'backlog' | 'todo' | 'inprogress' | 'done';
type SyncStatus = 'synced' | 'local_only' | 'modified';

type ItemRow = {
  id: string;
  user_id: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  sync_status: SyncStatus;
  due: string;
  duration_minutes: number | null;
  motivation: number | null;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
};

type AuthContext = {
  userId: string;
  token: string;
};

type AppEnv = { Variables: { auth: AuthContext } };

const app = new Hono<AppEnv>();

const corsAllowAll = process.env.CORS_ALLOW_ALL === 'true';
const defaultLocalOrigins = [
  'http://localhost:5173',
  'http://127.0.0.1:5173',
  'http://localhost:3000',
  'http://127.0.0.1:3000',
];

const configuredCorsOrigins = (process.env.CORS_ALLOWED_ORIGINS ?? '')
  .split(',')
  .map((origin) => origin.trim())
  .filter((origin) => origin.length > 0);

const corsAllowedOrigins = Array.from(new Set([...defaultLocalOrigins, ...configuredCorsOrigins]));

function isOriginAllowed(origin: string | undefined): boolean {
  if (!origin) return true;
  if (corsAllowAll) return true;
  return corsAllowedOrigins.includes(origin);
}

app.use('/api/*', async (c, next) => {
  const origin = c.req.header('Origin');
  if (!isOriginAllowed(origin)) {
    return c.json({ error: 'CORS origin forbidden' }, 403);
  }
  await next();
});

app.use('/api/*', cors({
  origin: (origin) => {
    if (!origin) return '*';
    if (corsAllowAll) return origin;
    return corsAllowedOrigins.includes(origin) ? origin : '';
  },
  credentials: true,
}));

function parseBearerToken(header: string | undefined): string | null {
  if (!header) return null;
  const [scheme, token] = header.split(' ');
  if (scheme?.toLowerCase() !== 'bearer' || !token) return null;
  return token;
}

function createSupabaseWithToken(token: string) {
  return createClient(SUPABASE_URL!, SUPABASE_ANON_KEY!, {
    global: {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    },
    auth: {
      persistSession: false,
      autoRefreshToken: false,
    },
  });
}

function createAnonSupabase() {
  return createClient(SUPABASE_URL!, SUPABASE_ANON_KEY!, {
    auth: {
      persistSession: false,
      autoRefreshToken: false,
    },
  });
}

function normalizeIso(value: string | null): string | null {
  if (value === null) return null;
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return value;
  return d.toISOString();
}

function normalizeItem(row: ItemRow): ItemRow {
  return {
    ...row,
    due: normalizeIso(row.due) ?? row.due,
    created_at: normalizeIso(row.created_at) ?? row.created_at,
    updated_at: normalizeIso(row.updated_at) ?? row.updated_at,
    deleted_at: normalizeIso(row.deleted_at),
  };
}

function parseStatus(raw: unknown): TaskStatus | null {
  if (typeof raw !== 'string') return null;
  const value = raw.toLowerCase();
  if (value === 'backlog') return 'backlog';
  if (value === 'todo') return 'todo';
  if (value === 'inprogress') return 'inprogress';
  if (value === 'done') return 'done';
  return null;
}

function isUuid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}

async function parseJson<T>(c: Context, fallback: T): Promise<T> {
  try {
    const parsed = await c.req.json<T>();
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
      return { ...fallback, ...(parsed as any) };
    }
    return parsed;
  } catch {
    return fallback;
  }
}

const requireAuth: MiddlewareHandler<AppEnv> = async (c, next) => {
  const token = parseBearerToken(c.req.header('Authorization'));
  if (!token) {
    return c.json({ error: 'Missing or invalid Authorization header' }, 401);
  }

  const anon = createAnonSupabase();
  const { data, error } = await anon.auth.getUser(token);
  if (error || !data.user) {
    return c.json({ error: 'Unauthorized' }, 401);
  }

  c.set('auth', { userId: data.user.id, token });
  await next();
};

async function fetchProfileUsername(token: string, userId: string): Promise<string> {
  const supabase = createSupabaseWithToken(token);
  const { data, error } = await supabase
    .from('profiles')
    .select('username')
    .eq('id', userId)
    .maybeSingle();

  if (error) {
    return 'Unknown User';
  }

  const username = data?.username?.trim();
  return username ? username : 'Unknown User';
}

async function handleGetActiveItems(c: Context<AppEnv>): Promise<Response> {
  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { data, error } = await supabase
    .from('items')
    .select('*')
    .is('deleted_at', null)
    .eq('is_archived', false)
    .order('due', { ascending: true, nullsFirst: false })
    .order('created_at', { ascending: false });

  if (error) return c.json({ error: error.message }, 400);
  return c.json((data ?? []).map((row) => normalizeItem(row as ItemRow)));
}

async function handleGetArchivedItems(c: Context<AppEnv>): Promise<Response> {
  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { data, error } = await supabase
    .from('items')
    .select('*')
    .is('deleted_at', null)
    .eq('is_archived', true)
    .order('created_at', { ascending: false });

  if (error) return c.json({ error: error.message }, 400);
  return c.json((data ?? []).map((row) => normalizeItem(row as ItemRow)));
}

async function handleGetDeletedItems(c: Context<AppEnv>): Promise<Response> {
  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { data, error } = await supabase
    .from('items')
    .select('*')
    .not('deleted_at', 'is', null)
    .order('deleted_at', { ascending: false });

  if (error) return c.json({ error: error.message }, 400);
  return c.json((data ?? []).map((row) => normalizeItem(row as ItemRow)));
}

async function handleCreateItem(c: Context<AppEnv>): Promise<Response> {
  try {
    const { userId, token } = c.get('auth');
    const supabase = createSupabaseWithToken(token);
    const body = await parseJson(c, {
      id: '',
      title: '',
      description: null as string | null,
      motivation: null as number | null,
      due: '',
      durationMinutes: null as number | null,
      duration_minutes: null as number | null,
    });

    const title = typeof body.title === 'string' ? body.title.trim() : '';
    const motivation =
      typeof body.motivation === 'number' && Number.isFinite(body.motivation)
        ? body.motivation
        : null;
    const due = typeof body.due === 'string' ? body.due.trim() : '';
    const rawDuration = body.duration_minutes ?? body.durationMinutes;
    const durationMinutes =
      typeof rawDuration === 'number' && Number.isFinite(rawDuration)
        ? rawDuration
        : null;

    if (!title || !due) {
      return c.json({ error: 'title and due are required' }, 400);
    }

    const { data, error } = await supabase
      .from('items')
      .insert({
        id: body.id || crypto.randomUUID(),
        user_id: userId,
        title,
        description: body.description,
        status: 'todo',
        sync_status: 'synced',
        due,
        duration_minutes: durationMinutes,
        motivation,
        is_archived: false,
      })
      .select('id')
      .single();

    if (error) return c.json({ error: error.message }, 400);
    return c.json({ id: data.id });
  } catch {
    return c.json({ error: 'Failed to create item' }, 500);
  }
}

async function handleUpdateItemStatus(c: Context<AppEnv>): Promise<Response> {
  const idFromPath = c.req.param('id');
  const body = await parseJson(c, { id: '', status: '' });

  const id = typeof idFromPath === 'string' && idFromPath.trim().length > 0 ? idFromPath : body.id;

  if (!id || !body.status) {
    return c.json({ error: 'id and status are required' }, 400);
  }

  const validatedStatus = parseStatus(body.status);
  if (!validatedStatus) {
    return c.json({ error: 'invalid status' }, 400);
  }

  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { error } = await supabase
    .from('items')
    .update({ 
      status: validatedStatus,
      updated_at: new Date().toISOString()
    })
    .eq('id', id)
    .is('deleted_at', null);

  if (error) return c.json({ error: error.message }, 400);
  return c.body(null, 204);
}

async function handleUpdateItem(c: Context<AppEnv>): Promise<Response> {
  const idFromPath = c.req.param('id');
  const body = await parseJson(c, {
    id: '',
    title: '',
    description: null as string | null,
    motivation: null as number | null,
    due: '',
    durationMinutes: null as number | null,
    duration_minutes: null as number | null,
  });

  const id = typeof idFromPath === 'string' && idFromPath.trim().length > 0 ? idFromPath : body.id;
  if (!id || !isUuid(id)) {
    return c.json({ error: 'valid item id is required' }, 400);
  }

  const title = typeof body.title === 'string' ? body.title.trim() : '';
  const due = typeof body.due === 'string' ? body.due.trim() : '';
  if (!title || !due || Number.isNaN(Date.parse(due))) {
    return c.json({ error: 'valid title and due are required' }, 400);
  }

  const description =
    body.description === null || typeof body.description === 'string'
      ? body.description
      : null;
  const motivation =
    typeof body.motivation === 'number' && Number.isFinite(body.motivation)
      ? body.motivation
      : null;
  const rawDuration = body.duration_minutes ?? body.durationMinutes;
  const durationMinutes =
    typeof rawDuration === 'number' && Number.isFinite(rawDuration)
      ? rawDuration
      : null;

  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { error } = await supabase
    .from('items')
    .update({
      title,
      description,
      motivation,
      due: new Date(due).toISOString(),
      duration_minutes: durationMinutes,
      sync_status: 'modified',
      updated_at: new Date().toISOString(),
    })
    .eq('id', id)
    .is('deleted_at', null);

  if (error) return c.json({ error: error.message }, 400);
  return c.body(null, 204);
}

async function handleArchiveItem(c: Context<AppEnv>): Promise<Response> {
  const idFromPath = c.req.param('id');
  const body = await parseJson(c, { id: '' });

  const id = typeof idFromPath === 'string' && idFromPath.trim().length > 0 ? idFromPath : body.id;
  if (!id || !isUuid(id)) return c.json({ error: 'valid id is required' }, 400);

  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { error } = await supabase
    .from('items')
    .update({ is_archived: true, updated_at: new Date().toISOString() })
    .eq('id', id)
    .is('deleted_at', null);

  if (error) return c.json({ error: error.message }, 400);
  return c.body(null, 204);
}

async function handleSoftDeleteItem(c: Context<AppEnv>): Promise<Response> {
  const idFromPath = c.req.param('id');
  const body = await parseJson(c, { id: '' });

  const id = typeof idFromPath === 'string' && idFromPath.trim().length > 0 ? idFromPath : body.id;
  if (!id || !isUuid(id)) return c.json({ error: 'valid id is required' }, 400);

  const { token } = c.get('auth');
  const supabase = createSupabaseWithToken(token);

  const { error } = await supabase
    .from('items')
    .update({ deleted_at: new Date().toISOString(), updated_at: new Date().toISOString() })
    .eq('id', id)
    .is('deleted_at', null);

  if (error) return c.json({ error: error.message }, 400);
  return c.body(null, 204);
}

app.get('/api/hello', (c) => {
  return c.json({
    message: 'Hello from Hono (Backend)!',
    timestamp: new Date().toISOString(),
  });
});

// Auth endpoints
app.post('/api/auth/login', async (c) => {
  const body = await parseJson(c, { email: '', password: '' });
  const email = typeof body.email === 'string' ? body.email.trim() : '';
  const password = typeof body.password === 'string' ? body.password : '';

  if (!email || !password) {
    return c.json({ error: 'Email and password are required' }, 400);
  }

  const anon = createAnonSupabase();
  const { data, error } = await anon.auth.signInWithPassword({ email, password });

  if (error || !data.user || !data.session) {
    return c.json({ error: error?.message ?? 'Supabase auth failed' }, 401);
  }

  const username = await fetchProfileUsername(data.session.access_token, data.user.id);

  return c.json({
    id: data.user.id,
    username,
    access_token: data.session.access_token,
    refresh_token: data.session.refresh_token,
    expires_at: data.session.expires_at,
  });
});

app.post('/api/auth/logout', (c) => c.body(null, 204));

app.get('/api/auth/session', requireAuth, async (c) => {
  const { userId, token } = c.get('auth');
  const username = await fetchProfileUsername(token, userId);

  return c.json({
    id: userId,
    user_id: userId,
    username,
    last_login: null,
    is_active: 1,
  });
});

app.post('/api/auth/auto-login', requireAuth, async (c) => {
  const { userId, token } = c.get('auth');
  const username = await fetchProfileUsername(token, userId);

  return c.json({
    id: userId,
    user_id: userId,
    username,
    last_login: null,
    is_active: 1,
  });
});

app.use('/api/items/*', requireAuth);
app.use('/api/commands/*', requireAuth);

// Items endpoints
app.get('/api/items/active', async (c) => handleGetActiveItems(c));

app.get('/api/items/archived', async (c) => handleGetArchivedItems(c));

app.get('/api/items/deleted', async (c) => handleGetDeletedItems(c));

app.post('/api/items', async (c) => handleCreateItem(c));

app.post('/api/items/create', async (c) => handleCreateItem(c));

app.patch('/api/items/:id', async (c) => handleUpdateItem(c));

app.post('/api/items/update', async (c) => handleUpdateItem(c));

app.patch('/api/items/:id/status', async (c) => handleUpdateItemStatus(c));

app.post('/api/items/update-status', async (c) => handleUpdateItemStatus(c));

app.post('/api/items/:id/archive', async (c) => handleArchiveItem(c));

app.post('/api/items/archive', async (c) => handleArchiveItem(c));

app.delete('/api/items/:id', async (c) => handleSoftDeleteItem(c));

app.post('/api/items/soft-delete', async (c) => handleSoftDeleteItem(c));

app.post('/api/items/sync', (c) => c.json({ count: 0 }));

// Tauri command aliases
app.get('/api/commands/get_active_items', async (c) => handleGetActiveItems(c));

app.get('/api/commands/get_archived_items', async (c) => handleGetArchivedItems(c));

app.get('/api/commands/get_deleted_items', async (c) => handleGetDeletedItems(c));

app.post('/api/commands/create_item', async (c) => handleCreateItem(c));

app.post('/api/commands/update_item_status', async (c) => handleUpdateItemStatus(c));

app.patch('/api/commands/update_item/:id', async (c) => handleUpdateItem(c));

app.post('/api/commands/update_item_details', async (c) => handleUpdateItem(c));

app.post('/api/commands/archive_item', async (c) => handleArchiveItem(c));

app.post('/api/commands/soft_delete_item', async (c) => handleSoftDeleteItem(c));

app.post('/api/commands/sync_items', (c) => c.json({ count: 0 }));

serve({
  fetch: app.fetch,
  port: Number(port),
  hostname: '0.0.0.0'
}, (info) => {
  console.log(`Server is listening on http://0.0.0.0:${info.port}`);
});
