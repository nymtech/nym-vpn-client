param (
    [Parameter(Mandatory = $true)][string]$build_configuration,
    [Parameter(Mandatory = $true)][string]$platform
)

write-output "Compiling winfw in $build_configuration for $platform"

MSBuild.exe /m .\nym-vpn-windows\winfw\winfw.sln /p:Configuration=$build_configuration /p:Platform=$platform
