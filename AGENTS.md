# AGENTS.md — SelfHost Helper

Electron desktop app for managing local dev projects (processes, monitoring, editor, tunnels). React + Vite renderer, Electron main process with native C++ addon.

## Dev Commands

```bash
npm run dev          # starts Vite (port 5173) + Electron concurrently
npm run dev:react    # Vite only (renderer)
npm run dev:electron # Electron only (expects Vite running)
npm run lint         # ESLint
npm run typecheck    # tsc --noEmit
npm run format       # Prettier write
npm run format:check # Prettier check
npm run build        # vite build + electron-builder (full desktop build)
npm run build:web    # vite build only (renderer dist/)
```

CI order: `npm run lint` → `npm run typecheck` → `npm run build:web`. No test suite exists.

## Path Alias

`@` → `src/` (configured in both `vite.config.js` and `jsconfig.json`). Use `@/components/Foo` not relative `../../components/Foo`.

## Project Structure

```
electron/           # Electron main process (Node.js, not bundled by Vite)
  main.js           # Entry point, window creation, protocol handler, startup
  preload.js        # contextBridge API exposed to renderer as window.api
  ipc/handlers.js   # IPC handler registration
  services/         # All backend logic (17 services)
  tray/             # System tray
  job/              # Native C++ addon (Windows Job Objects) — node-gyp, excluded from tsconfig/eslint
src/                # React renderer (Vite-bundled)
  main.jsx          # React entry
  App.jsx           # Router (HashRouter), routes
  components/       # UI components (21 files)
  pages/            # Dashboard, Settings, settings sections
  store/            # Jotai atoms (state management)
  hooks/            # Custom hooks
  lib/              # Shared utilities
  config/           # File tag config
  editors/          # Monaco editor wrappers
  monacoWorkers.js  # Monaco worker setup
database/models/    # Sequelize models (Category.js, Project.js) — SQLite
```

## Architecture

- **Two-process split**: Electron main (`electron/`) and React renderer (`src/`). They communicate via IPC (`preload.js` defines the `window.api` bridge).
- **State**: Jotai atoms in `src/store/atoms.js`. No Redux.
- **Database**: SQLite via Sequelize. Models in `database/models/`. DB file at `src/Database/Data/Database.sqlite` (gitignored).
- **Native addon**: `electron/job/` is a C++ node-gyp addon for Windows Job Objects (process cleanup). It has its own `package.json` and builds automatically via `postinstall`. Excluded from tsconfig and ESLint.
- **Custom protocol**: `media://` protocol registered in `electron/main.js` for serving local images to the renderer with path validation and allowlisting.
- **Routing**: HashRouter (`react-router-dom`). Routes defined in `App.jsx`.

## Code Conventions

- **Language**: Plain JavaScript (JSX), not TypeScript. `tsconfig.json` exists for type-checking only (`noEmit: true`, `strict: false`).
- **Formatting**: Prettier — double quotes, 2-space indent, trailing commas (es5), 100 char print width.
- **Linting**: ESLint with React/hooks plugins. Several base rules disabled (`no-undef: off`, `no-unused-vars: off`) — existing code relies on this.
- **Tailwind**: v4 with `@tailwindcss/postcss` plugin. shadcn/ui-style CSS variables (HSL-based color tokens).
- **UI library**: Radix UI primitives + `class-variance-authority` + `tailwind-merge` (shadcn pattern). Components in `src/components/ui/`.
- **No semicolons enforced** — Prettier keeps them (`semi: true`).

## Gotchas

- `npm install` requires `legacy-peer-deps=true` (set in `.npmrc`).
- `electron/job/` native addon: if it fails to build, run `cd electron/job && npm run build` (needs node-gyp + C++ toolchain).
- Dev mode sets a separate `userData` path (`userData-dev`) so dev and prod databases don't collide.
- Dev mode loads `react-scan` from unpkg — if offline, it silently fails.
- `no-undef` is off in ESLint — undefined vars won't lint-error. Be careful with typos.
- ESLint ignores `electron/job/**` and `.github/**`.
- Prettier ignores `electron/job/build`, `public/file-icons`, `release*`, `dist`, `build`.
- The Vite server is on strict port 5173 (`strictPort: true`). If occupied, `npm run dev:electron` will retry up to 5 times then show a fallback error page.
- `.env` files are gitignored. `publish:prod` script uses `dotenv-cli` to load `.env` for publish tokens.
