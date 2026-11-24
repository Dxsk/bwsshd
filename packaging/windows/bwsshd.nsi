; Per-user setup (no administrator rights): copies bwsshd.exe to
; %LOCALAPPDATA%\Programs\bwsshd and runs `bwsshd install`, which registers the
; logon task. Built by the release workflow:
;
;   makensis -DVERSION=1.2.3 -DBIN=path\to\bwsshd.exe -DOUT=bwsshd_setup.exe packaging/windows/bwsshd.nsi

Unicode true
RequestExecutionLevel user
SetCompressor /SOLID lzma

!define UNINST "Software\Microsoft\Windows\CurrentVersion\Uninstall\bwsshd"

Name "bwsshd ${VERSION}"
OutFile "${OUT}"
InstallDir "$LOCALAPPDATA\Programs\bwsshd"
ShowInstDetails show
ShowUninstDetails show

Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section
  ; The running daemon locks its exe.
  nsExec::Exec '"$SYSDIR\schtasks.exe" /End /TN bwsshd'
  Pop $0
  Sleep 1000

  SetOutPath "$INSTDIR"
  File "/oname=bwsshd.exe" "${BIN}"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  WriteRegStr HKCU "${UNINST}" "DisplayName" "bwsshd"
  WriteRegStr HKCU "${UNINST}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINST}" "Publisher" "Dxsk"
  WriteRegStr HKCU "${UNINST}" "URLInfoAbout" "https://github.com/Dxsk/bwsshd"
  WriteRegStr HKCU "${UNINST}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINST}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINST}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINST}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST}" "NoRepair" 1

  nsExec::ExecToLog '"$INSTDIR\bwsshd.exe" install'
  Pop $0
  StrCmp $0 0 +3
    SetErrorLevel 1
    Abort "bwsshd install failed ($0)"
SectionEnd

Section "Uninstall"
  nsExec::ExecToLog '"$INSTDIR\bwsshd.exe" uninstall'
  Pop $0
  Sleep 1000
  Delete "$INSTDIR\bwsshd.exe"
  Delete "$INSTDIR\uninstall.exe"
  ; The tray panel's WebView2 data, created next to the exe.
  RMDir /r "$INSTDIR\bwsshd.exe.WebView2"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNINST}"
SectionEnd
