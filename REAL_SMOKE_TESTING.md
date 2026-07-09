# Real Web Smoke Testing

Run from the project root:

```powershell
.\scripts\test-web-real-smoke.ps1
```

The script starts a disposable PostgreSQL container from `docker-compose.smoke.yml`, starts the Rust API on `127.0.0.1:18080`, starts Vite on `127.0.0.1:3100`, then runs Playwright against the real API.

The real smoke flow checks:

- default admin login through the real backend;
- squad and equipment creation through real API endpoints;
- web login in the browser;
- equipment list rendering from real API data;
- booking creation from the web UI;
- booking visibility through the real API and UI.

Useful overrides:

```powershell
$env:SMOKE_API_PORT = "18080"
$env:SMOKE_WEB_PORT = "3100"
$env:SMOKE_DATABASE_URL = "postgresql://postgres:postgres@127.0.0.1:55432/kvantoriym_smoke"
$env:REAL_SMOKE_ADMIN_USERNAME = "admin"
$env:REAL_SMOKE_ADMIN_PASSWORD = "smoke-admin-password"
.\scripts\test-web-real-smoke.ps1
```

If PostgreSQL is already running and you do not want Docker orchestration:

```powershell
$env:SMOKE_DATABASE_URL = "postgresql://postgres:postgres@127.0.0.1:5432/kvantoriym"
.\scripts\test-web-real-smoke.ps1 -SkipDocker
```
