import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

function readEnv(
  name: string,
  fileEnv: Record<string, string>,
  fallback: string,
): string {
  const fromProcess = process.env[name]?.trim()
  if (fromProcess) return fromProcess
  const fromFile = fileEnv[name]?.trim()
  if (fromFile) return fromFile
  return fallback
}

export default defineConfig(({ mode }) => {
  const fileEnv = loadEnv(mode, process.cwd(), '')
  const apiHost = readEnv('API_HOST', fileEnv, '127.0.0.1')
  const apiPort = readEnv('API_PORT', fileEnv, '3000')
  const frontendHost = readEnv('FRONTEND_HOST', fileEnv, '127.0.0.1')
  const frontendPort = Number(readEnv('FRONTEND_PORT', fileEnv, '5173'))
  const targetHost = apiHost.includes(':') ? `[${apiHost}]` : apiHost

  return {
    plugins: [vue()],
    server: {
      host: frontendHost,
      port: frontendPort,
      strictPort: true,
      proxy: {
        '/api': {
          target: `http://${targetHost}:${apiPort}`,
          changeOrigin: true,
        },
      },
    },
  }
})
