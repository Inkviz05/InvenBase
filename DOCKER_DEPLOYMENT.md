# Docker Deployment

This compose profile runs the production-shaped local stack:

- PostgreSQL 16
- Rust backend API
- React web build served by nginx
- nginx reverse proxy for `/api/*` and `/health`

## Quick Start

Create a local env file:

```powershell
Copy-Item .env.docker.example .env
```

Edit `.env` and set strong values for:

- `POSTGRES_PASSWORD`
- `JWT_SECRET`
- `DEFAULT_ADMIN_PASSWORD`

Start the stack:

```powershell
docker compose up --build
```

Open:

```text
http://localhost
```

The backend API is available through the reverse proxy:

```text
http://localhost/api
```

Health check:

```text
http://localhost/health
```

## First Admin

For the first local launch you can keep:

```env
CREATE_DEFAULT_ADMIN=true
```

After the first admin is created, set it back to:

```env
CREATE_DEFAULT_ADMIN=false
```

The backend will not overwrite an existing admin with the same username, but disabling bootstrap after initial setup is safer.

## Data

PostgreSQL data is stored in the named Docker volume:

```text
postgres_data
```

To stop containers without deleting data:

```powershell
docker compose down
```

To delete containers and database volume:

```powershell
docker compose down -v
```

## Services

The public entrypoint is the `web` service on `WEB_PORT` from `.env`.

Internal services:

- `backend:8080`
- `postgres:5432`

The web image is built with:

```env
VITE_API_URL=/api
```

This keeps browser API calls on the same origin as the web app and lets nginx proxy them to the backend container.

## Smoke Tests

The separate real smoke test stack remains available:

```powershell
.\scripts\test-web-real-smoke.ps1
```

It uses `docker-compose.smoke.yml` and a disposable PostgreSQL database on port `55432`.
