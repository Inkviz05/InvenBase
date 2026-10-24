param(
    [Parameter(Mandatory = $true)]
    [string]$BackupPath,
    [Parameter(Mandatory = $true)]
    [string]$TargetDatabaseUrl,
    [switch]$ConfirmRestore,
    [switch]$AllowNonTestDatabase
)

$ErrorActionPreference = "Stop"

function Get-PostgresConnection {
    param([string]$Url)

    $uri = [Uri]$Url
    if ($uri.Scheme -notin @("postgres", "postgresql")) {
        throw "Target database URL must use postgres:// or postgresql://"
    }

    $credentials = $uri.UserInfo.Split(":", 2)
    if ($credentials.Count -ne 2) {
        throw "Target database URL must contain username and password"
    }

    $database = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart("/"))
    if ($database -notmatch "^[A-Za-z0-9_-]+$") {
        throw "Target database name contains unsupported characters"
    }

    [PSCustomObject]@{
        Host = $uri.Host
        Port = if ($uri.Port -gt 0) { $uri.Port } else { 5432 }
        Username = [Uri]::UnescapeDataString($credentials[0])
        Password = [Uri]::UnescapeDataString($credentials[1])
        Database = $database
    }
}

if (-not $ConfirmRestore) {
    throw "Restore replaces objects in the target database. Pass -ConfirmRestore to continue."
}

$resolvedBackupPath = [IO.Path]::GetFullPath($BackupPath)
if (-not (Test-Path -LiteralPath $resolvedBackupPath -PathType Leaf)) {
    throw "Backup file does not exist: $resolvedBackupPath"
}

foreach ($command in @("pg_restore", "psql", "createdb")) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "$command is not installed or is not available in PATH"
    }
}

$connection = Get-PostgresConnection -Url $TargetDatabaseUrl
if (-not $AllowNonTestDatabase -and $connection.Database -notmatch "(?i)(test|restore|staging)") {
    throw "Refusing to restore into '$($connection.Database)'. Use a test/restore/staging database name or pass -AllowNonTestDatabase explicitly."
}

$previousPassword = $env:PGPASSWORD
$env:PGPASSWORD = $connection.Password
try {
    & pg_restore --list $resolvedBackupPath | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Backup is not a readable pg_restore archive: $resolvedBackupPath"
    }

    $exists = & psql `
        --host=$($connection.Host) `
        --port=$($connection.Port) `
        --username=$($connection.Username) `
        --dbname=postgres `
        --tuples-only `
        --no-align `
        --command="SELECT 1 FROM pg_database WHERE datname = '$($connection.Database)'"
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to inspect target PostgreSQL instance"
    }

    if (($exists | Out-String).Trim() -ne "1") {
        & createdb `
            --host=$($connection.Host) `
            --port=$($connection.Port) `
            --username=$($connection.Username) `
            $connection.Database
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create target database '$($connection.Database)'"
        }
    }

    & pg_restore `
        --host=$($connection.Host) `
        --port=$($connection.Port) `
        --username=$($connection.Username) `
        --dbname=$($connection.Database) `
        --exit-on-error `
        --clean `
        --if-exists `
        --no-owner `
        --no-acl `
        $resolvedBackupPath
    if ($LASTEXITCODE -ne 0) {
        throw "pg_restore failed with exit code $LASTEXITCODE"
    }

    $migrationCount = & psql `
        --host=$($connection.Host) `
        --port=$($connection.Port) `
        --username=$($connection.Username) `
        --dbname=$($connection.Database) `
        --tuples-only `
        --no-align `
        --command="SELECT COUNT(*) FROM _sqlx_migrations"
    if ($LASTEXITCODE -ne 0 -or [int](($migrationCount | Out-String).Trim()) -lt 1) {
        throw "Restored database does not contain SQLx migration history"
    }
} finally {
    $env:PGPASSWORD = $previousPassword
}

Write-Output "Restore completed: $($connection.Database)"
