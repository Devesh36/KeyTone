$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$releaseApi = "https://api.github.com/repos/Devesh36/KeyTone/releases/latest"
$temporaryDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("keytone-" + [System.Guid]::NewGuid())

try {
    Write-Host "Keytone: Finding the latest release..."
    $release = Invoke-RestMethod -Uri $releaseApi -Headers @{ Accept = "application/vnd.github+json" }
    $asset = $release.assets |
        Where-Object { $_.name -match "_x64_en-US\.msi$" } |
        Select-Object -First 1

    if (-not $asset) {
        $asset = $release.assets |
            Where-Object { $_.name -match "_x64-setup\.exe$" } |
            Select-Object -First 1
    }

    if (-not $asset) {
        throw "No Windows installer was found in the latest release."
    }

    New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
    $installerPath = Join-Path $temporaryDirectory $asset.name
    Write-Host "Keytone: Downloading $($asset.name)..."
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $installerPath

    Write-Host "Keytone: Starting the installer..."
    $process = Start-Process -FilePath $installerPath -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "The installer exited with code $($process.ExitCode)."
    }

    Write-Host "Keytone: Installation complete."
}
finally {
    if (Test-Path $temporaryDirectory) {
        Remove-Item -Recurse -Force $temporaryDirectory
    }
}
