<#
.SYNOPSIS
Downloads the Windows MIDI Services SDK NuGet package and extracts the metadata and runtime
files Rumpus builds against into the sdk/ folder.
#>
[CmdletBinding()]
param(
	[string]$Release = "inbox-dev-preview-8",
	[string]$Package = "Windows.Devices.Midi2.0.99.75-devpreview.8.nupkg"
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$sdkDir = Join-Path $repoRoot "sdk"
$nupkg = Join-Path $sdkDir $Package
$url = "https://github.com/microsoft/MIDI/releases/download/$Release/$Package"

New-Item -ItemType Directory -Force -Path $sdkDir | Out-Null
if (-not (Test-Path $nupkg)) {
	Write-Host "Downloading $url"
	Invoke-WebRequest -Uri $url -OutFile $nupkg
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($nupkg)
try {
	$wanted = @(
		"ref/native/Windows.Devices.Midi2.winmd",
		"runtimes/win-x64/native/Windows.Devices.Midi2.dll",
		"runtimes/win-x64/native/Windows.Devices.Midi2.pri",
		"runtimes/win-arm64/native/Windows.Devices.Midi2.dll",
		"runtimes/win-arm64/native/Windows.Devices.Midi2.pri"
	)
	foreach ($name in $wanted) {
		$entry = $zip.GetEntry($name)
		if ($null -eq $entry) { throw "$name not found in $Package" }
		$target = if ($name.StartsWith("ref/")) {
			Join-Path $sdkDir (Split-Path -Leaf $name)
		} else {
			Join-Path $sdkDir ($name -replace "/", [IO.Path]::DirectorySeparatorChar)
		}
		New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
		[System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target, $true)
		Write-Host "Extracted $target"
	}
} finally {
	$zip.Dispose()
}
