function Get-CodesTag {
    param(
        [string]$Repo = "4fuu/code-search-cli",
        [string]$Version = "latest"
    )

    if ($Version -eq "latest") {
        $resp = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        return $resp.tag_name
    }

    if ($Version.StartsWith("v")) {
        return $Version
    }

    return "v$Version"
}

function Install-Codes {
    [CmdletBinding()]
    param(
        [string]$Version = "latest",
        [string]$Repo = "4fuu/code-search-cli",
        [string]$InstallDir = "$HOME\AppData\Local\Programs\codes\bin"
    )

    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -ne "AMD64") {
        throw "Only Windows x86_64 is currently supported by the PowerShell installer."
    }

    $tag = Get-CodesTag -Repo $Repo -Version $Version
    $asset = "codes-$tag-x86_64-pc-windows-msvc.zip"
    $base = "https://github.com/$Repo/releases/download/$tag"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("codes-install-" + [guid]::NewGuid().ToString("N"))
    $zipPath = Join-Path $tmp $asset
    $sumPath = Join-Path $tmp "SHA256SUMS.txt"

    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    try {
        Invoke-WebRequest -Uri "$base/$asset" -OutFile $zipPath
        Invoke-WebRequest -Uri "$base/SHA256SUMS.txt" -OutFile $sumPath

        $expected = (Select-String -Path $sumPath -Pattern ([regex]::Escape($asset)) | Select-Object -First 1).Line.Split()[0]
        $actual = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($expected.ToLowerInvariant() -ne $actual) {
            throw "Checksum mismatch for $asset"
        }

        Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
        Copy-Item (Join-Path $tmp "codes.exe") (Join-Path $InstallDir "codes.exe") -Force

        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if (-not ($userPath -split ';' | Where-Object { $_ -eq $InstallDir })) {
            $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $InstallDir } else { "$userPath;$InstallDir" }
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        }

        Write-Host "Installed codes to $InstallDir\codes.exe"
        Write-Host "Open a new terminal if PATH has not refreshed yet."
    }
    finally {
        Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}
