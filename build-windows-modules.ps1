param (
    [Parameter(Mandatory = $true)]
    [ValidateSet('Release', 'Debug')]
    # Build configuration
    [string]$BuildConfiguration,

    [Parameter(Mandatory = $true)]
    [ValidateSet('x64', 'ARM64')]
    # CPU architecture
    [string]$Platform,

    # Whether to copy compiled binaries to build\winfw
    # In debug builds this also makes sure to copy winfw.dll to nym-vpn-core\target\debug
    [bool]$CopyToBuildDir = $False
)

Write-Output "Compiling winfw in $BuildConfiguration for $Platform"

MSBuild.exe /m .\nym-vpn-windows\winfw\winfw.sln /p:Configuration=$BuildConfiguration /p:Platform=$Platform

if ($CopyToBuildDir) {
    $BuildDir = "$PSScriptRoot\nym-vpn-windows\winfw\bin\$Platform-$BuildConfiguration"
    $OutputDir = "$PSScriptRoot\build\winfw\$Platform-$BuildConfiguration"
    $RustTargetDir = "$PSScriptRoot\nym-vpn-core\target"
    $RustBuildDir = "$RustTargetDir\$($BuildConfiguration.ToLower())"
    $BaseLibPath = "$BuildDir\winfw"

    # Copy winfw.{lib,dll} to build/libwf
    Write-Output "Copying winfw.{lib,dll} to $OutputDir"
    New-Item -ItemType Directory -Force -Path $OutputDir -Verbose
    Copy-Item -Path "$BaseLibPath.lib" -Destination $OutputDir -Verbose
    Copy-Item -Path "$BaseLibPath.dll" -Destination $OutputDir -Verbose

    # Copy winfw.dll to target/<profile>
    New-Item -ItemType Directory -Force -Path $RustBuildDir -Verbose
    Copy-Item -Path "$BaseLibPath.dll" -Destination $RustBuildDir -Verbose
}
