param(
    [switch]$Staged
)

$ErrorActionPreference = "Stop"

if ($Staged) {
    $files = @(git diff --cached --name-only --diff-filter=ACMR)
    $scope = "staged files"
} else {
    $files = @(git ls-files)
    $scope = "tracked files"
}

if ($LASTEXITCODE -ne 0) {
    throw "Unable to read $scope from Git"
}

$forbiddenNames = @(
    "google-services.json",
    "agconnect-services.json",
    "firebase-service-account.json"
)
$forbiddenExtensions = @(".pem", ".key", ".p12", ".pfx", ".jks", ".keystore")
$contentPatterns = @(
    "-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
    '"private_key"\s*:',
    '"client_secret"\s*:',
    "(?m)^\s*//registry\.npmjs\.org/:_authToken\s*="
)

$violations = [System.Collections.Generic.List[string]]::new()

foreach ($path in $files) {
    if ([string]::IsNullOrWhiteSpace($path)) {
        continue
    }

    if (-not $Staged -and -not (Test-Path -LiteralPath $path -PathType Leaf)) {
        continue
    }

    $normalized = $path.Replace("\", "/")
    $name = [System.IO.Path]::GetFileName($normalized).ToLowerInvariant()
    $extension = [System.IO.Path]::GetExtension($name).ToLowerInvariant()

    $isEnvironmentSecret = $name -eq ".env" -or
        ($name.StartsWith(".env.") -and -not $name.EndsWith(".example"))
    $isServiceAccount = $name -like "*service-account*.json"

    if ($isEnvironmentSecret -or
        $forbiddenNames -contains $name -or
        $isServiceAccount -or
        $forbiddenExtensions -contains $extension) {
        $violations.Add("$normalized (forbidden secret/config filename)")
        continue
    }

    try {
        if ($Staged) {
            $content = (git show ":$path") -join "`n"
            if ($LASTEXITCODE -ne 0) {
                throw "git show failed"
            }
        } else {
            $content = Get-Content -LiteralPath $path -Raw -ErrorAction Stop
        }
    } catch {
        # Binary and unreadable files are still covered by filename rules and Gitleaks.
        continue
    }

    foreach ($pattern in $contentPatterns) {
        if ($content -match $pattern) {
            $violations.Add("$normalized (secret-like content)")
            break
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Error ("Secret check rejected {0}:`n - {1}" -f $scope, ($violations -join "`n - "))
    exit 1
}

Write-Host "Secret check passed for $scope."
