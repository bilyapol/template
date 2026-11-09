# React Frontend

The frontend is a single-page application built with React 19, TypeScript, Vite, and Tailwind CSS.

## Project Structure

```
frontend/
├── index.html               # Entry HTML (Vite injects scripts)
├── vite.config.ts            # Dev server, /api proxy, Tailwind plugin
├── tsconfig.json             # TypeScript project references
├── package.json              # Dependencies and scripts
└── src/
    ├── main.tsx              # React root mount
    ├── App.tsx               # Routes and route guards
    ├── index.css             # Tailwind import + theme tokens
    ├── api/                  # HTTP client layer
    │   ├── client.ts         # apiFetch() base function
    │   ├── auth.ts           # Login, logout, auth mode
    │   └── docs.ts           # In-app docs
    ├── stores/               # Zustand state management
    │   ├── auth.store.ts     # Current user state
    │   └── authmode.store.ts # Google / dev / none login mode
    ├── types/                # TypeScript interfaces
    │   └── api.ts            # User type
    └── components/
        ├── layout/           # App shell, header
        │   ├── AppShell.tsx  # Auth bootstrap, loading state
        │   └── Header.tsx    # Nav bar with login/logout
        ├── pages/            # Route-level components
        │   ├── HomePage.tsx  # Landing page with feature cards
        │   ├── DashboardPage.tsx  # Empty authenticated page
        │   └── DocPage.tsx   # Documentation viewer
        └── ui/               # Shared UI components
            └── Spinner.tsx   # Loading spinner
```

## How It Works

### Vite Dev Server

During development, Vite serves the frontend on port 5173 and proxies all `/api` requests to the Rust backend on port 3000:

```typescript
// vite.config.ts
server: {
  port: 5173,
  proxy: {
    "/api": {
      target: "http://localhost:3000",
      changeOrigin: true,
    },
  },
},
```

In production, the Axum backend serves the built frontend files directly — no separate web server needed.

### API Client

All HTTP calls go through a single `apiFetch` function that handles credentials, JSON headers, and error parsing:

```typescript
// api/client.ts
export async function apiFetch<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const res = await fetch(`${path}`, {
    credentials: "include",  // sends auth cookie
    headers: { "Content-Type": "application/json", ...opts.headers },
    ...opts,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new ApiError(res.status, body.error || "Unknown error");
  }
  return res.json();
}
```

Resource-specific API functions are thin wrappers:

```typescript
// api/widgets.ts
export function fetchWidgets(): Promise<Widget[]> {
  return apiFetch<Widget[]>("/api/widgets");
}

export function createWidget(data: { title: string }): Promise<Widget> {
  return apiFetch<Widget>("/api/widgets", {
    method: "POST",
    body: JSON.stringify(data),
  });
}
```

### State Management (Zustand)

Each domain has its own Zustand store — lightweight, no boilerplate:

```typescript
// stores/widgets.store.ts
export const useWidgetsStore = create<WidgetsState>((set, get) => ({
  widgets: [],
  isLoading: false,
  load: async () => {
    set({ isLoading: true });
    const widgets = await api.fetchWidgets();
    set({ widgets, isLoading: false });
  },
  add: async (title) => {
    const widget = await api.createWidget({ title });
    set({ widgets: [widget, ...get().widgets] });
  },
}));
```

Use in components:

```tsx
const { widgets, load, add } = useWidgetsStore();
useEffect(() => { load(); }, [load]);
```

### Route Guards

Protected routes use a `RequireAuth` wrapper that redirects unauthenticated users:

```tsx
function RequireAuth({ children }: { children: ReactNode }) {
  const user = useAuthStore((s) => s.user);
  if (!user) return <Navigate to="/" replace />;
  return <>{children}</>;
}

// Usage in routes:
<Route path="/dashboard" element={<RequireAuth><DashboardPage /></RequireAuth>} />
```

### Auth Bootstrap

The `AppShell` component runs on mount:
1. Fetches `GET /api/auth/mode` to determine login method (`google`, `dev`, or `none` in production without OAuth)
2. Fetches `GET /api/user/me` — if the JWT cookie is valid, user state is populated
3. While loading, shows a spinner; then renders the app

### Tailwind Theme

Colors and fonts are defined as CSS custom properties via Tailwind's `@theme` directive:

```css
@theme {
  --color-bg: #0F172A;
  --color-surface: #1E293B;
  --color-accent: #F97316;
  --color-text: #F1F5F9;
  /* ... */
}
```

Use them in classes: `bg-surface`, `text-accent`, `border-border-subtle`, etc.

### Path Aliases

The `@/` alias maps to `src/`, so imports are clean:

```typescript
import { useAuthStore } from "@/stores/auth.store";
import { apiFetch } from "@/api/client";
```

## Adding a New Page

1. Create a component: `frontend/src/components/pages/MyPage.tsx`
2. Add a route in `App.tsx`: `<Route path="/my-page" element={<MyPage />} />`
3. Wrap with `<RequireAuth>` if it needs authentication
4. Add a link in `Header.tsx` if it should appear in navigation

## Adding a New Store

1. Create `frontend/src/stores/mydata.store.ts`
2. Define the state interface and create the store with `create<MyState>(...)`
3. Add API functions in `frontend/src/api/mydata.ts`
4. Use in components: `const { data, load } = useMyDataStore()`

## Scripts

```bash
npm run dev      # Start Vite dev server (port 5173)
npm run build    # Type-check + production build to dist/
npm run lint     # Run ESLint
npm run preview  # Preview production build locally
```
