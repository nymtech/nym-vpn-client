; NSIS installer hooks, see https://v2.tauri.app/distribute/windows-installer/#installer-hooks
;
; This file is included near the top of the upstream template, before most
; of its `!define`s (e.g. `ARCH`). Anything depending on those must live
; inside the NSIS_HOOK_* macros, which are expanded later.

${StrTok}
${UnStrTok}

!define APPDIR "nym-vpn-app"

!define MUI_HEADERIMAGE_RIGHT

Var VpndVersion
Var VpndVersionMajor
Var VpndVersionMinor

; It's a NSIS thing that any function called from the Uninstall section
; must be prefixed with `un.`. Cherry on top of the cake, any function
; called from a `un.` func, must be `un.` too (:

Function GetVpndSemVer
  ${StrTok} $VpndVersionMajor "$VpndVersion" "." "0" "1"
  ${StrTok} $VpndVersionMinor "$VpndVersion" "." "1" "1"
FunctionEnd

; un. version of above, plain dup
Function un.GetVpndSemVer
  ${UnStrTok} $VpndVersionMajor "$VpndVersion" "." "0" "1"
  ${UnStrTok} $VpndVersionMinor "$VpndVersion" "." "1" "1"
FunctionEnd

; Here we use macros to avoid code dup
; From install section, it can be called as `GetVpndVersionFull`
; From uninstall section, it can be called as `un.GetVpndVersionFull`
!macro VPND_GET_VERSION_FULL un
  Function ${un}GetVpndVersionFull
    nsExec::ExecToStack '"$INSTDIR\nym-vpnd.exe" -V'
    Pop $0
    Pop $1
    StrCpy $VpndVersion "$1" "" 9
    DetailPrint "vpnd version: $VpndVersion"
    Call ${un}GetVpndSemVer
    DetailPrint "vpnd semver major: $VpndVersionMajor"
    DetailPrint "vpnd semver minor: $VpndVersionMinor"
  FunctionEnd
!macroend

; Same here we use macros to avoid code dup
; From install section, call it with `VpndUninstall`
; From uninstall section, call it with `un.VpndUninstall`
!macro VPND_UNINSTALL un
  Function ${un}VpndUninstall
    Call ${un}GetVpndVersionFull
    vpnd-uninstall:
      ${If} $VpndVersionMajor == 1
      ${AndIf} $VpndVersionMinor < 14
        DetailPrint "vpnd version is pre 1.14, using --uninstall flag"
        ExecWait '"$INSTDIR\nym-vpnd.exe" --uninstall' $9
      ${Else}
        DetailPrint "Waiting 30 seconds for the vpnd service to stop and uninstall"
        ExecWait '"$INSTDIR\nym-vpnd.exe" uninstall-service' $9
      ${EndIf}
    ${If} $9 <> 0
      ; if vpnd fails to uninstall all we can do is to notify the user
      ; !DO NOT ABORT HERE! this will lock forever the user with a broken uninstaller!
      ; preventing user to install/uninstall/update
      DetailPrint "'nym-vpnd.exe uninstall-service' failed: $9"
      MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Failed to uninstall NymVPN service [$9]" /SD IDABORT IDRETRY vpnd-uninstall
    ${EndIf}
  FunctionEnd
!macroend

!insertmacro VPND_GET_VERSION_FULL ""
!insertmacro VPND_GET_VERSION_FULL "un."
!insertmacro VPND_UNINSTALL ""
!insertmacro VPND_UNINSTALL "un."

; Same file list for install, cleanup and uninstall
!macro VPND_FILES op
  !insertmacro ${op} "nym-vpnd.exe"
  !insertmacro ${op} "nym-socks5-proxy.exe"
  !insertmacro ${op} "libwg.dll"
  !insertmacro ${op} "wintun.dll"
  !insertmacro ${op} "winfw.dll"
  !insertmacro ${op} "nymvpn-split-tunnel.sys"
  !insertmacro ${op} "nymvpn-split-tunnel.inf"
  !insertmacro ${op} "nymvpn-split-tunnel.cat"
!macroend

!macro VPND_FILE_COPY name
  File "${RESPREFIX}\${name}"
!macroend

!macro VPND_FILE_DELETE name
  Delete "$INSTDIR\${name}"
!macroend

; Clear the command line arguments of a .lnk shortcut, no-op if it doesn't exist
;
; Shortcuts created by installers up to 2026.10 pass `-l -Ldebug`. `-l` is not
; a real flag: the app accepts it only as a hidden no-op (`Cli::log_file`) so
; those shortcuts keep launching.
;
; Clearing them here does not make that shim removable. The updater relays the
; old process's arguments to the new binary through /ARGS (see .onInstSuccess),
; and taskbar and start menu pins are separate .lnk files this never sees, so
; `-l` keeps reaching the binary from places the installer cannot reach.
;
; An update skips shortcut creation altogether (see $UpdateMode), so stale
; arguments are never overwritten by the normal path and have to be cleared
; explicitly.
;
; `utils.nsh` has no equivalent: its SetShortcutTarget sets only the target
; path (IShellLink::SetPath) and leaves the arguments alone.
;
; This clears every argument, not just `-l`. The app is not meant to be
; launched from a shortcut with arguments, so anything found there is discarded.
;
; Note this modifies $0, $1 and $2
!macro ClearShortcutArgs shortcut
  ${If} ${FileExists} "${shortcut}"
    !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ShellLink} ${IID_IShellLink} r0 ""
    ${If} $0 P<> 0
      ${IUnknown::QueryInterface} $0 '("${IID_IPersistFile}",.r1)'
      ${If} $1 P<> 0
        ; STGM_READWRITE asks for exclusive access, so Load fails if anything
        ; else holds the .lnk. Saving anyway would write an empty shortcut over
        ; a working one, since nothing has set a target on this IShellLink.
        ${IPersistFile::Load} $1 '("${shortcut}", ${STGM_READWRITE})i.r2'
        ${If} $2 >= 0
          ${IShellLink::SetArguments} $0 '(w "")'
          ${IPersistFile::Save} $1 '("${shortcut}",1)'
          DetailPrint "cleared shortcut arguments: ${shortcut}"
        ${Else}
          DetailPrint "could not open shortcut [$2]: ${shortcut}"
        ${EndIf}
        ${IUnknown::Release} $1 ""
      ${EndIf}
      ${IUnknown::Release} $0 ""
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREINSTALL
  ; The template checks for a running app only after this hook. Check it before
  ; removing the service, so that cancelling it leaves the existing install intact.
  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

  ; Stop and remove the service of an existing install before overwriting
  ; its files (update, reinstall, or up/downgrade without uninstalling)
  ${If} ${FileExists} "$INSTDIR\nym-vpnd.exe"
    Call VpndUninstall
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; ARM64 builds place the NSIS script one directory level deeper than x64
  ; (target/<triple>/release/nsis/arm64/ vs target/release/nsis/x86-64/),
  ; so the relative path to src-tauri/ differs by one level.
  !if "${ARCH}" == "arm64"
    !define RESPREFIX "..\..\..\..\.."
  !else
    !define RESPREFIX "..\..\..\.."
  !endif

  ; Copy vpnd, socks5-proxy, libs and split-tunnel driver artifacts
  !insertmacro VPND_FILES VPND_FILE_COPY

  !undef RESPREFIX

  ; Install NymVPN service, abort early if something goes wrong
  vpnd-install:
    ExecWait '"$INSTDIR\nym-vpnd.exe" install-service' $3
  ${If} $3 <> 0
    DetailPrint "'nym-vpnd.exe install-service' failed: $3"
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "An error occurred while installing NymVPN service [$3]" /SD IDABORT IDRETRY vpnd-install
      ; The template has already registered the uninstaller, which needs
      ; nym-vpnd.exe to remove a partially installed service, so keep the
      ; daemon files and only remove the app so it can't run without them
      Delete "$INSTDIR\${MAINBINARYNAME}.exe"
      Abort
  ${EndIf}

  ; Start NymVPN service, notify user if something goes wrong
  vpnd-start:
    ExecWait '"$INSTDIR\nym-vpnd.exe" start-service' $3
  ${If} $3 <> 0
    ; if vpnd fails to start we notify the user and continue as this
    ; can been solved later on
    DetailPrint "'nym-vpnd.exe start-service' failed: $3"
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "Failed to start NymVPN service [$3]" /SD IDABORT IDRETRY vpnd-start
  ${EndIf}

  ; Shortcuts have been created or updated by now. The desktop one can also be
  ; created later by the finish page checkbox, which is fine: that one is new.
  !insertmacro ClearShortcutArgs "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !insertmacro ClearShortcutArgs "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !insertmacro ClearShortcutArgs "$DESKTOP\${PRODUCTNAME}.lnk"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; As on install, the template checks for a running app only after this hook
  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

  ; Must run before the template's non-recursive `RMDir "$INSTDIR"`
  Call un.VpndUninstall
  !insertmacro VPND_FILES VPND_FILE_DELETE
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    RmDir /r "$APPDATA\${APPDIR}"
    RmDir /r "$LOCALAPPDATA\${APPDIR}"

    ; Switch context to point to <DRIVE>:\ProgramData
    SetShellVarContext all
    RmDir /r "$APPDATA\nym-vpnd"
    ; Revert to current context just in case
    SetShellVarContext current
  ${EndIf}
!macroend
