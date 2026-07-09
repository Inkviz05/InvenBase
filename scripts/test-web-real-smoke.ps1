param(
    [switch]$SkipDocker,
    [switch]$KeepDocker
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$serverDir = Join-Path $repoRoot "invenbase-server"
$webDir = Join-Path $repoRoot "invenbase-client\web"
$composeFile = Join-Path $repoRoot "docker-compose.smoke.yml"

$apiPort = if ($env:SMOKE_API_PORT) { $env:SMOKE_API_PORT } else { "18080" }
$webPort = if ($env:SMOKE_WEB_PORT) { $env:SMOKE_WEB_PORT } else { "3100" }
$databaseUrl = if ($env:SMOKE_DATABASE_URL) { $env:SMOKE_DATABASE_URL } else { "postgresql://postgres:postgres@127.0.0.1:55432/kvantoriym_smoke" }
$adminUsername = if ($env:REAL_SMOKE_ADMIN_USERNAME) { $env:REAL_SMOKE_ADMIN_USERNAME } else { "admin" }
$adminPassword = if ($env:REAL_SMOKE_ADMIN_PASSWORD) { $env:REAL_SMOKE_ADMIN_PASSWORD } else { "smoke-admin-password" }
$jwtSecret = if ($env:REAL_SMOKE_JWT_SECRET) { $env:REAL_SMOKE_JWT_SECRET } else { "real-smoke-jwt-secret-minimum-32-chars" }

function Assert-CommandExists {
    param([string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "$Name is not installed or is not available in PATH"
    }
}

function Wait-ForApi {
    param(
        [string]$BaseUrl,
        [string]$Username,
        [string]$Password
    )

    $body = @{ username = $Username; password = $Password } | ConvertTo-Json

    for ($i = 0; $i -lt 90; $i++) {
        try {
            Invoke-RestMethod `
                -Method Post `
                -Uri "$BaseUrl/api/auth/login" `
                -ContentType "application/json" `
                -Body $body `
                -TimeoutSec 3 | Out-Null
            return
        } catch {
            Start-Sleep -Seconds 2
        }
    }

    throw "Backend did not become ready at $BaseUrl"
}

if (-not $SkipDocker) {
    Assert-CommandExists "docker"
    docker compose -f $composeFile up -d postgres-smoke
}

$backendEnv = @{
    DATABASE_URL = $databaseUrl
    HOST = "127.0.0.1"
    PORT = $apiPort
    JWT_SECRET = $jwtSecret
    JWT_EXPIRATION = "86400"
    CREATE_DEFAULT_ADMIN = "true"
    DEFAULT_ADMIN_USERNAME = $adminUsername
    DEFAULT_ADMIN_PASSWORD = $adminPassword
    DEFAULT_ADMIN_EMAIL = "admin@kvantoriym.local"
    DEFAULT_ADMIN_FULL_NAME = "Smoke Admin"
    CORS_ALLOWED_ORIGINS = "http://127.0.0.1:$webPort,http://localhost:$webPort"
    RUST_LOG = "info"
}

$backendJob = $null

try {
    $backendJob = Start-Job -Name "invenbase-real-smoke-api" -ScriptBlock {
        param($WorkingDirectory, $Environment)

        Set-Location -LiteralPath $WorkingDirectory
        foreach ($key in $Environment.Keys) {
            Set-Item -Path "Env:$key" -Value $Environment[$key]
        }

        cargo run
    } -ArgumentList $serverDir, $backendEnv

    Wait-ForApi -BaseUrl "http://127.0.0.1:$apiPort" -Username $adminUsername -Password $adminPassword

    Push-Location $webDir
    try {
        $env:SMOKE_WEB_PORT = $webPort
        $env:VITE_API_PORT = $apiPort
        $env:REAL_SMOKE_ADMIN_USERNAME = $adminUsername
        $env:REAL_SMOKE_ADMIN_PASSWORD = $adminPassword
        npm run test:e2e:real-smoke
    } finally {
        Pop-Location
    }
} finally {
    if ($backendJob) {
        Stop-Job -Job $backendJob -ErrorAction SilentlyContinue
        Remove-Job -Job $backendJob -Force -ErrorAction SilentlyContinue
    }

    if (-not $SkipDocker -and -not $KeepDocker) {
        docker compose -f $composeFile down -v
    }
}
