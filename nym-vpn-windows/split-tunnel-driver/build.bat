@echo off

if [%VisualStudioVersion%]==[] (
  echo Please launch this build script from a Visual Studio command prompt
  exit /b 1
)

if [%1]==[] goto USAGE

set CERT_THUMBPRINT=%1
set TIMESTAMP_SERVER=http://timestamp.digicert.com
set CAB_ARCH=%VSCMD_ARG_TGT_ARCH%
if "%CAB_ARCH%"=="x64" (
  set CAB_ARCH=amd64
)

set INF2CAT_OS=10_%VSCMD_ARG_TGT_ARCH%
if "%VSCMD_ARG_TGT_ARCH%"=="arm64" (
  set INF2CAT_OS=10_VB_%VSCMD_ARG_TGT_ARCH%
)

set ROOT=%~dp0

:: Force complete rebuild

rmdir /s /q %ROOT%bin

:: Build driver but do not sign it
:: It's not possible to control all arguments to signtool through msbuild

msbuild.exe %ROOT%nymvpn-split-tunnel\nymvpn-split-tunnel.vcxproj /p:Configuration=Release /p:Platform=%VSCMD_ARG_TGT_ARCH% /p:SignMode=Off

IF %ERRORLEVEL% NEQ 0 goto ERROR

:: Sign driver

signtool sign /tr %TIMESTAMP_SERVER% /td sha256 /fd sha256 /sha1 "%CERT_THUMBPRINT%" /v %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.sys

IF %ERRORLEVEL% NEQ 0 goto ERROR

:: Re-generate catalog file now that driver binary has changed

del %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.cat
"%WindowsSdkVerBinPath%x86\inf2cat.exe" /driver:%ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel /os:"%INF2CAT_OS%" /verbose

IF %ERRORLEVEL% NEQ 0 goto ERROR

:: Sign catalog

signtool sign /tr %TIMESTAMP_SERVER% /td sha256 /fd sha256 /sha1 "%CERT_THUMBPRINT%" /v %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.cat

IF %ERRORLEVEL% NEQ 0 goto ERROR

:: Build a CAB file for submission to the MS Hardware Dev Center

mkdir %ROOT%bin\temp\cab

>"%ROOT%bin\temp\cab\nymvpn-split-tunnel-%CAB_ARCH%.ddf" (
    echo .OPTION EXPLICIT     ; Generate errors
    echo .Set CabinetFileCountThreshold=0
    echo .Set FolderFileCountThreshold=0
    echo .Set FolderSizeThreshold=0
    echo .Set MaxCabinetSize=0
    echo .Set MaxDiskFileCount=0
    echo .Set MaxDiskSize=0
    echo .Set CompressionType=MSZIP
    echo .Set Cabinet=on
    echo .Set Compress=on
    echo .Set CabinetNameTemplate=nymvpn-split-tunnel-%CAB_ARCH%.cab
    echo .Set DestinationDir=Package
    echo .Set DiskDirectoryTemplate=%ROOT%bin\temp\cab
    echo %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.cat
    echo %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.inf
    echo %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.sys
    echo %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.pdb
)

:: makecab produces several garbage files
:: Force current working directory to prevent spreading them out

pushd %ROOT%bin\temp\cab

makecab /f "%ROOT%bin\temp\cab\nymvpn-split-tunnel-%CAB_ARCH%.ddf"

popd

IF %ERRORLEVEL% NEQ 0 goto ERROR

signtool sign /tr %TIMESTAMP_SERVER% /td sha256 /fd sha256 /sha1 "%CERT_THUMBPRINT%" /v %ROOT%bin\temp\cab\nymvpn-split-tunnel-%CAB_ARCH%.cab

IF %ERRORLEVEL% NEQ 0 goto ERROR

:: Collect artifacts

mkdir %ROOT%bin\dist

copy /b %ROOT%bin\%VSCMD_ARG_TGT_ARCH%-Release\nymvpn-split-tunnel\nymvpn-split-tunnel.pdb %ROOT%bin\dist\
copy /b %ROOT%bin\temp\cab\nymvpn-split-tunnel-%CAB_ARCH%.cab %ROOT%bin\dist\

echo;
echo BUILD COMPLETED SUCCESSFULLY
echo ARTIFACTS ARE IN --^> bin/dist/ ^<--
echo;

exit /b 0

:USAGE

echo Usage: %0 ^<cert_sha1_hash^>
exit /b 1

:ERROR

echo;
echo !!! BUILD FAILED !!!
echo;

exit /b 1
