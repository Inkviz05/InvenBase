param(
    [switch]$SkipBuild,
    [switch]$KeepDocker
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$webDir = Join-Path $repoRoot "invenbase-client/web"
$composeFile = Join-Path $repoRoot "docker-compose.yml"
$projectName = if ($env:COMPOSE_SMOKE_PROJECT_NAME) { $env:COMPOSE_SMOKE_PROJECT_NAME } else { "invenbase-smoke" }
$webPort = if ($env:SMOKE_WEB_PORT) { $env:SMOKE_WEB_PORT } else { "3100" }
$adminUsername = if ($env:REAL_SMOKE_ADMIN_USERNAME) { $env:REAL_SMOKE_ADMIN_USERNAME } else { "admin" }
$adminPassword = if ($env:REAL_SMOKE_ADMIN_PASSWORD) { $env:REAL_SMOKE_ADMIN_PASSWORD } else { "smoke-admin-password" }

function Assert-CommandExists {
    param([string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name is not installed or is not available in PATH"
    }
}

function Wait-ForStack {
    param([string]$BaseUrl)

    for ($i = 0; $i -lt 90; $i++) {
        try {
            $response = Invoke-WebRequest -Uri "$BaseUrl/health" -TimeoutSec 3
            if ($response.StatusCode -eq 200) {
                return
            }
        } catch {
            Start-Sleep -Seconds 2
        }
    }

    docker compose -p $projectName -f $composeFile ps
    docker compose -p $projectName -f $composeFile logs --no-color
    throw "Docker Compose stack did not become healthy at $BaseUrl"
}

Assert-CommandExists "docker"
Assert-CommandExists "npm"

$env:POSTGRES_DB = "kvantoriym_smoke"
$env:POSTGRES_USER = "postgres"
$env:POSTGRES_PASSWORD = "smoke-postgres-password"
$env:JWT_SECRET = "compose-smoke-jwt-secret-minimum-32-characters"
$env:JWT_EXPIRATION = "86400"
$env:CREATE_DEFAULT_ADMIN = "true"
$env:DEFAULT_ADMIN_USERNAME = $adminUsername
$env:DEFAULT_ADMIN_PASSWORD = $adminPassword
$env:DEFAULT_ADMIN_EMAIL = "admin@kvantoriym.local"
$env:DEFAULT_ADMIN_FULL_NAME = "Smoke Admin"
$env:CORS_ALLOWED_ORIGINS = "http://127.0.0.1:$webPort,http://localhost:$webPort"
$env:WEB_PORT = $webPort
$env:RUST_LOG = "info"

$baseUrl = "http://127.0.0.1:$webPort"

try {
    $composeArguments = @("compose", "-p", $projectName, "-f", $composeFile, "up", "-d")
    if (-not $SkipBuild) {
        $composeArguments += "--build"
    }
    & docker @composeArguments
    if ($LASTEXITCODE -ne 0) {
        throw "docker compose up failed with exit code $LASTEXITCODE"
    }

    Wait-ForStack -BaseUrl $baseUrl

    Push-Location $webDir
    try {
        $env:DOCKER_SMOKE_BASE_URL = $baseUrl
        $env:REAL_SMOKE_ADMIN_USERNAME = $adminUsername
        $env:REAL_SMOKE_ADMIN_PASSWORD = $adminPassword
        npm run test:e2e:docker-smoke
        if ($LASTEXITCODE -ne 0) {
            throw "Playwright smoke test failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
} finally {
    if (-not $KeepDocker) {
        docker compose -p $projectName -f $composeFile down -v --remove-orphans
    }
}
