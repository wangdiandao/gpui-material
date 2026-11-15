#Requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $Executable,

    [Parameter(Mandatory)]
    [string] $OutputDirectory
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$target = 'x86_64-pc-windows-msvc'
$gallery = 'gpui-material-gallery'
$executablePath = (Get-Item -LiteralPath $Executable).FullName
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $outputPath) {
    throw "Choose a new output directory: $outputPath"
}

Push-Location (Join-Path $PSScriptRoot '..')
$stage = $null
try {
    $commit = git rev-parse HEAD
    $rust = rustc -vV
    $tree = cargo tree --locked --offline --package $gallery --target $target `
        --edges normal,no-proc-macro --prefix none --format '{p}'
    $metadata = cargo metadata --locked --offline --filter-platform $target --format-version 1 |
        ConvertFrom-Json -Depth 100

    $active = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($line in $tree) {
        if ($line -notmatch '^(\S+) v(\S+)') {
            throw "Unexpected Cargo tree entry: $line"
        }
        [void] $active.Add("$($Matches[1])@$($Matches[2])")
    }

    $stage = [IO.Directory]::CreateTempSubdirectory('material-gallery-')
    $bundle = New-Item -ItemType Directory (Join-Path $stage.FullName 'bundle')
    $licenses = New-Item -ItemType Directory (Join-Path $bundle.FullName 'licenses')
    Copy-Item -LiteralPath $executablePath -Destination (Join-Path $bundle.FullName "$gallery.exe")
    Copy-Item -LiteralPath 'LICENSE', 'THIRD_PARTY_NOTICES.md', 'THIRD_PARTY_NOTICES.zh.md' `
        -Destination $bundle.FullName
    Copy-Item -LiteralPath 'crates/components/assets/material-symbols/LICENSE' `
        -Destination (Join-Path $licenses.FullName 'Material-Symbols-LICENSE')

    $supplement = @{
        'accesskit@0.24.1' = 'accesskit'
        'accesskit_consumer@0.38.0' = 'accesskit'
        'accesskit_windows@0.34.0' = 'accesskit'
        'lyon@1.0.19' = 'lyon'
        'lyon_algorithms@1.0.22' = 'lyon'
        'lyon_geom@1.0.20' = 'lyon'
        'lyon_path@1.0.19' = 'lyon'
        'lyon_tessellation@1.0.23' = 'lyon'
        'profiling@1.0.18' = 'profiling'
        'seahash@4.1.0' = 'seahash'
        'svg_fmt@0.4.5' = 'svg_fmt'
        'taffy@0.15.1' = 'taffy'
        'zune-inflate@0.2.54' = 'zune-inflate'
        'gpui-pre-windows@0.3.12' = 'gpui-windows'
    }
    $registryCount = 0
    $inventory = foreach ($package in ($metadata.packages | Sort-Object name, version)) {
        $id = "$($package.name)@$($package.version)"
        if (!$active.Remove($id) -or !$package.source) {
            continue
        }
        $registryCount++
        $root = Split-Path $package.manifest_path
        $files = @(Get-ChildItem -LiteralPath $root -File |
            Where-Object Name -Match '^(licen[cs]es?|copying|copyright|notice|unlicense)(\b|[._-])' |
            ForEach-Object FullName)
        if ($package.license_file) {
            $files += Join-Path $root $package.license_file
        }
        if ($supplement.ContainsKey($id)) {
            $supplementDir = Join-Path $PSScriptRoot "gallery-licenses/$($supplement[$id])"
            $files += Get-ChildItem -LiteralPath $supplementDir -File | ForEach-Object FullName
        }
        # mac declares MIT or Apache-2.0 and ships no separate license text.
        if ($id -eq 'mac@0.1.1') {
            $files += (Get-Item -LiteralPath LICENSE).FullName
        }
        if (!$files) {
            throw "No redistribution license material found for $id"
        }

        $folder = "$($package.name)-$($package.version)"
        $destination = New-Item -ItemType Directory (Join-Path $licenses.FullName $folder)
        Copy-Item -LiteralPath ($files | Select-Object -Unique) -Destination $destination.FullName
        "$id | $($package.license) | $($package.source) | licenses/$folder/"
    }
    if ($active.Count -ne 0) {
        throw "Cargo metadata did not include: $($active -join ', ')"
    }
    $inventory | Set-Content -LiteralPath (Join-Path $bundle.FullName 'DEPENDENCIES.txt')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'gallery-license-sources.txt') -Destination $licenses.FullName

    $rustDocs = Join-Path (rustc --print sysroot) 'share/doc/rust'
    $rustLicenses = New-Item -ItemType Directory (Join-Path $licenses.FullName 'rust-std')
    Copy-Item -LiteralPath (Join-Path $rustDocs 'COPYRIGHT-library.html') -Destination $rustLicenses.FullName
    Copy-Item -LiteralPath (Join-Path $rustDocs 'licenses') -Destination $rustLicenses.FullName -Recurse

    @(
        "Commit: $commit"
        "Target: $target"
        'Profile: gallery-artifact'
        $rust
    ) | Set-Content -LiteralPath (Join-Path $bundle.FullName 'BUILD.txt')

    $archiveName = "$gallery-windows-x64.zip"
    $stagedArchive = Join-Path $stage.FullName $archiveName
    [IO.Compression.ZipFile]::CreateFromDirectory($bundle.FullName, $stagedArchive)
    $sha256 = (Get-FileHash -LiteralPath $stagedArchive).Hash.ToLowerInvariant()
    $checksums = Join-Path $stage.FullName 'SHA256SUMS.txt'
    "$sha256  $archiveName" | Set-Content -LiteralPath $checksums -Encoding ascii

    New-Item -ItemType Directory -Path $outputPath | Out-Null
    Copy-Item -LiteralPath $stagedArchive, $checksums -Destination $outputPath
    Write-Output "Packaged $registryCount registry dependencies in $(Join-Path $outputPath $archiveName)"
} finally {
    if ($stage) {
        $stage.Delete($true)
    }
    Pop-Location
}
