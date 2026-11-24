# bwsshd installer for Windows.
#
# Downloads the latest release for this machine, checks it against the
# published checksums, then runs the install.ps1 bundled in the archive: it
# copies bwsshd.exe to %LOCALAPPDATA%\Programs\bwsshd and registers the
# per-user logon task. No administrator rights needed.
#
#   irm https://bwsshd.example/install.ps1 | iex

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = 'Dxsk/bwsshd'
$isArm = $env:PROCESSOR_ARCHITECTURE -eq 'ARM64' -or $env:PROCESSOR_ARCHITEW6432 -eq 'ARM64'
$arch = if ($isArm) { 'arm64' } else { 'amd64' }

$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest" -Headers @{ 'User-Agent' = 'bwsshd-installer' }
$name = "bwsshd_$($release.tag_name.TrimStart('v'))_windows_$arch"
$asset = $release.assets | Where-Object name -eq "$name.zip"
if (-not $asset) { throw "bwsshd: no $name.zip in release $($release.tag_name)" }

$tmp = Join-Path ([IO.Path]::GetTempPath()) "bwsshd-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    Write-Host "bwsshd: downloading $name"
    $zip = Join-Path $tmp "$name.zip"
    Invoke-WebRequest $asset.browser_download_url -OutFile $zip -UseBasicParsing

    $sums = $release.assets | Where-Object name -eq 'checksums.txt'
    if ($sums) {
        $sumsFile = Join-Path $tmp 'checksums.txt'
        Invoke-WebRequest $sums.browser_download_url -OutFile $sumsFile -UseBasicParsing
        $line = Get-Content $sumsFile | Where-Object { ($_ -split '\s+')[1] -eq "$name.zip" } | Select-Object -First 1
        if ($line) {
            $want = ($line -split '\s+')[0]
            $got = (Get-FileHash $zip -Algorithm SHA256).Hash
            if ($want -ne $got) { throw "bwsshd: checksum mismatch for $name.zip" }
        }
    }

    $dir = Join-Path $tmp 'x'
    Expand-Archive $zip -DestinationPath $dir
    $installer = Get-ChildItem $dir -Recurse -Filter install.ps1 | Select-Object -First 1
    if (-not $installer) { throw "bwsshd: the archive has no install.ps1" }

    # Run it in a child process: a script file is subject to the execution
    # policy, unlike this piped one, and Restricted is the client default.
    $shell = (Get-Process -Id $PID).Path
    & $shell -NoProfile -ExecutionPolicy Bypass -File $installer.FullName
    if ($LASTEXITCODE) { throw "bwsshd: install.ps1 failed with exit code $LASTEXITCODE" }
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
