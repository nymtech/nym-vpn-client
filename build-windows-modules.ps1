param (
    [Parameter(Mandatory = $true)][string]$BuildConfiguration,
    [Parameter(Mandatory = $true)][string]$Platform,
    [bool]$CopyToBuildDir = $False
)

Write-Output "Compiling winfw in $BuildConfiguration for $Platform"

MSBuild.exe /m .\nym-vpn-windows\winfw\winfw.sln /p:Configuration=$BuildConfiguration /p:Platform=$Platform

$BuildDir = "$PSScriptRoot\nym-vpn-windows\winfw\bin\$Platform-$BuildConfiguration"
$OutputDir = "$PSScriptRoot\build\winfw\$Platform-$BuildConfiguration"

if ($CopyToBuildDir) {
    Write-Output "Copying winfw.{lib,dll} to $OutputDir"
    New-Item -ItemType Directory -Force -Path $OutputDir -Verbose
    Copy-Item -Path $BuildDir\winfw.lib -Destination $OutputDir -Verbose
    Copy-Item -Path $BuildDir\winfw.dll -Destination $OutputDir -Verbose
}