param(
    [string]$DatabaseUrl = $env:DATABASE_URL,
    [string]$BackupDirectory = "",
    [string]$OutputPath = "",
    [int]$RetentionDays = 14
)

$ErrorActionPreference = "Stop"

function Get-PostgresConnection {
    param([string]$Url)

    if (-not $Url) {
        throw "DATABASE_URL or -DatabaseUrl is required"
    }

    $uri = [Uri]$Url
    if ($uri.Scheme -notin @("postgres", "postgresql")) {
        throw "Database URL must use postgres:// or postgresql://"
    }

    $credentials = $uri.UserInfo.Split(":", 2)
    if ($credentials.Count -ne 2) {
        throw "Database URL must contain username and password"
    }

    [PSCustomObject]@{
        Host = $uri.Host
        Port = if ($uri.Port -gt 0) { $uri.Port } else { 5432 }
        Username = [Uri]::UnescapeDataString($credentials[0])
        Password = [Uri]::UnescapeDataString($credentials[1])
        Database = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart("/"))
    }
}

foreach ($command in @("pg_dump", "pg_restore")) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "$command is not installed or is not available in PATH"
    }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $BackupDirectory) {
    $BackupDirectory = Join-Path $repoRoot "backups"
}
$backupDirectoryPath = [IO.Path]::GetFullPath($BackupDirectory)
[IO.Directory]::CreateDirectory($backupDirectoryPath) | Out-Null

if (-not $OutputPath) {
    $timestamp = [DateTime]::UtcNow.ToString("yyyyMMdd-HHmmss")
    $OutputPath = Join-Path $backupDirectoryPath "invenbase-$timestamp.dump"
}
$backupPath = [IO.Path]::GetFullPath($OutputPath)
$backupParent = Split-Path -Parent $backupPath
[IO.Directory]::CreateDirectory($backupParent) | Out-Null

$connection = Get-PostgresConnection -Url $DatabaseUrl
$previousPassword = $env:PGPASSWORD
$env:PGPASSWORD = $connection.Password
try {
    & pg_dump `
        --host=$($connection.Host) `
        --port=$($connection.Port) `
        --username=$($connection.Username) `
        --dbname=$($connection.Database) `
        --format=custom `
        --no-owner `
        --no-acl `
        --file=$backupPath
    if ($LASTEXITCODE -ne 0) {
        throw "pg_dump failed with exit code $LASTEXITCODE"
    }

    if (-not (Test-Path -LiteralPath $backupPath) -or (Get-Item -LiteralPath $backupPath).Length -eq 0) {
        throw "Backup file was not created or is empty: $backupPath"
    }

    & pg_restore --list $backupPath | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "pg_restore could not read backup: $backupPath"
    }
} finally {
    $env:PGPASSWORD = $previousPassword
}

if ($RetentionDays -gt 0) {
    $cutoff = [DateTime]::UtcNow.AddDays(-$RetentionDays)
    Get-ChildItem -LiteralPath $backupDirectoryPath -File -Filter "invenbase-*.dump" |
        Where-Object { $_.LastWriteTimeUtc -lt $cutoff -and $_.FullName -ne $backupPath } |
        Remove-Item -Force
}

Write-Output $backupPath
