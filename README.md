# Outer Loop Todos

A small to-do app. The interface is Vue 3 (Vite). The API is Rust (Axum) and stores todos in memory for the life of the process. You can create a todo, list them, toggle one complete, and delete one.

The UI calls `/api` on its own origin. In development, Vite forwards `/api` to the Rust API.

## Prerequisites

Install these before the steps below. Versions are the ones this README was checked against.

| Tool | Version |
| --- | --- |
| Rust (`rustc` and `cargo`) | 1.99.1 (stable). Crates in the lockfile need edition 2024, so a toolchain older than 1.85 will not compile. |
| Node.js | 22.23.4 |
| npm | 10.9.9 (the npm that ships with Node.js 22) |

```bash
rustc --version
cargo --version
node --version
npm --version
```

CI uses Rust stable and Node.js 22.

## Install

From a fresh clone:

```bash
git clone https://github.com/zkan/ai-outer-loop-demo.git
cd ai-outer-loop-demo
npm ci
```

`npm ci` installs the Vue app from `package-lock.json`. Cargo downloads the API crates on the first `cargo run` or `cargo test`.

## Start the API and the UI separately

Use two terminals. Start the API first and wait until it prints the listening line.

Terminal 1, API on http://127.0.0.1:3000:

```bash
cd backend
cargo run
```

Terminal 2, from the repo root, UI on http://127.0.0.1:5173:

```bash
npm run dev
```

Open http://127.0.0.1:5173. Leave both processes running while you use the app. Stop each one with Ctrl+C in its terminal.

## Start both with one command

From the repo root, after `npm ci`:

```bash
npm run dev:all
```

That runs `scripts/dev.sh`. It starts the API, waits until the API port accepts a connection, then starts Vite. Ctrl+C stops both. The UI is still http://127.0.0.1:5173.

The first run compiles the API, so the "Waiting for the API to listen..." line can sit there while Cargo downloads crates and builds.

## Environment variables

Set these in the environment of the process you are starting. Defaults:

| Variable | Default | Effect |
| --- | --- | --- |
| `API_HOST` | `127.0.0.1` | Address the API binds to, and the host Vite proxies to. An IP address, or the name `localhost`. |
| `API_PORT` | `3000` | Port the API binds to, and the port Vite proxies to. Integer from 1 to 65535. |
| `FRONTEND_HOST` | `127.0.0.1` | Address the Vite dev server binds to. |
| `FRONTEND_PORT` | `5173` | Port the Vite dev server binds to. Vite refuses to pick a different port if this one is taken. |

The browser keeps calling relative `/api/...`. To move the API, set `API_HOST` and `API_PORT` for both the API process and the Vite process. `npm run dev:all` exports the same values to both.

```bash
API_PORT=4000 FRONTEND_PORT=8080 npm run dev:all
```

That serves the UI at http://127.0.0.1:8080 and the API at http://127.0.0.1:4000.

When you start them separately, export the same `API_PORT` (and `API_HOST`, if you change it) in both terminals before `cargo run` and `npm run dev`.

## Tests

API, from `backend/`:

```bash
cd backend
cargo test --locked
```

UI, from the repo root:

```bash
npm test
```

There are no frontend unit tests yet. `npm test` runs Node's test runner (`node --test`) and succeeds with zero tests. `npm run build` typechecks the Vue app and writes `dist/`.

```bash
npm run build
```

## Troubleshooting

**The page shows `Request failed (500)`.** Vite is up and cannot reach the API: the API process is stopped, or the two processes are not using the same `API_HOST` and `API_PORT`. The dev server answers `/api` with 500 in that case. Start the API, wait for `listening on http://...`, then reload the page. With separate terminals, export the same `API_PORT` for both commands.

**`Address already in use` (API) or `Port 5173 is already in use` (Vite).** A previous API or Vite process is still bound. Stop it, or pick another port (`API_PORT`, `FRONTEND_PORT`) and use that port in the browser.

**`API_PORT must be an integer from 1 to 65535`.** `API_PORT` is empty or not a whole number in that range. Unset it to use 3000.

**`API_HOST must be an IP address or "localhost"`.** `API_HOST` is a hostname other than `localhost`. Use `127.0.0.1` or another IP.

**`npm run dev:all` says `node_modules is missing`.** Run `npm ci` from the repo root first.

**The first `cargo run` looks stuck.** Cargo is downloading crates and compiling. Wait for `AI Outer Loop todos API listening on http://...`. `npm run dev:all` waits up to 180 seconds for that port.

**Todos disappear after a restart.** The API keeps them in memory only. Stopping the API process drops them. A browser refresh does not, as long as that same API process is still running.

**`npm ci` or `npm run build` fails on Node.** Use Node.js 22. The install command is `npm ci`, which requires `package-lock.json` and does not update it.
