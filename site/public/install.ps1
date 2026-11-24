# bwsshd installer for Windows.
#
# Downloads the latest release for this machine, checks it against the
# published checksums, copies bwsshd.exe to %LOCALAPPDATA%\Programs\bwsshd and
# runs `bwsshd install`, which registers the per-user logon task. No
# administrator rights needed.
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
    $exe = Get-ChildItem $dir -Recurse -Filter bwsshd.exe | Select-Object -First 1
    if (-not $exe) { throw "bwsshd: the archive has no bwsshd.exe" }

    $installDir = Join-Path $env:LOCALAPPDATA 'Programs\bwsshd'
    $target = Join-Path $installDir 'bwsshd.exe'
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    # The running daemon locks its exe. Older installs started it through a
    # wscript launcher, so ending the task alone may leave it running.
    # Windows PowerShell turns redirected stderr into a terminating error here.
    try { schtasks /End /TN bwsshd 2>&1 | Out-Null } catch {}
    Get-Process bwsshd -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $target } | Stop-Process -Force
    Start-Sleep -Milliseconds 500
    Copy-Item $exe.FullName $target -Force
    Remove-Item (Join-Path $installDir 'bwsshd-launch.vbs') -ErrorAction SilentlyContinue
    Write-Host "bwsshd: installed $(& $target --version) to $target"

    & $target install
    if ($LASTEXITCODE) { throw "bwsshd: install failed with exit code $LASTEXITCODE" }
}
finally {
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
