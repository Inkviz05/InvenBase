param(
    [switch]$NoCapture
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$serverDir = Join-Path $repoRoot "invenbase-server"
$envPath = Join-Path $serverDir ".env"

function Read-DotEnvValue {
    param(
        [string]$Path,
        [string]$Name
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }

    foreach ($line in Get-Content -LiteralPath $Path) {
        $trimmed = $line.Trim()
        if ($trimmed.Length -eq 0 -or $trimmed.StartsWith("#")) {
            continue
        }

        $parts = $trimmed.Split("=", 2)
        if ($parts.Length -eq 2 -and $parts[0].Trim() -eq $Name) {
            return $parts[1].Trim().Trim('"').Trim("'")
        }
    }

    return $null
}

if (-not $env:TEST_DATABASE_URL) {
    $databaseUrl = Read-DotEnvValue -Path $envPath -Name "DATABASE_URL"
    if (-not $databaseUrl) {
        throw "TEST_DATABASE_URL is not set and DATABASE_URL was not found in invenbase-server/.env"
    }

    $env:TEST_DATABASE_URL = $databaseUrl
    Write-Host "TEST_DATABASE_URL was not set. Using DATABASE_URL from invenbase-server/.env"
}

Write-Host "Running backend tests against: $env:TEST_DATABASE_URL"

Push-Location $serverDir
try {
    if ($NoCapture) {
        cargo test -- --nocapture
    } else {
        cargo test
    }
} finally {
    Pop-Location
}
