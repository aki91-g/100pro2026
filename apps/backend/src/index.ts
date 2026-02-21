import { Hono } from 'hono'
import { cors } from 'hono/cors'
import { serve } from '@hono/node-server'

const app = new Hono()

// CORSを許可して、Frontendからのリクエストを受け付けるようにする
app.use('/api/*', cors())

app.get('/api/hello', (c) => {
  return c.json({
    message: 'Hello from Hono (Backend)!',
    timestamp: new Date().toISOString()
  })
})

serve({ fetch: app.fetch, port: 3000 })