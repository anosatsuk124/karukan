!include "MUI2.nsh"

Name "karukan Japanese Input Method"
!ifdef VERSION
  OutFile "Karukan-Setup-${VERSION}-x86_64.exe"
!else
  OutFile "Karukan-Setup-x86_64.exe"
!endif
InstallDir "$PROGRAMFILES64\karukan"
RequestExecutionLevel admin

; MUI pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "Japanese"
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath $INSTDIR
  File "karukan_tsf.dll"

  ; Register COM/TSF via DllRegisterServer
  ExecWait 'regsvr32 /s "$INSTDIR\karukan_tsf.dll"'

  ; Grant read+execute to AppContainer apps (UWP/modern apps)
  ExecWait 'icacls "$INSTDIR\karukan_tsf.dll" /grant *S-1-15-2-1:(RX)'

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Add/Remove Programs entry
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan" \
    "DisplayName" "karukan Japanese Input Method"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan" \
    "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan" \
    "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan" \
    "Publisher" "karukan"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan" \
    "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan" \
    "NoRepair" 1
SectionEnd

Section "Uninstall"
  ; Unregister COM/TSF via DllUnregisterServer
  ExecWait 'regsvr32 /u /s "$INSTDIR\karukan_tsf.dll"'

  Delete "$INSTDIR\karukan_tsf.dll"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  ; Remove Add/Remove Programs entry
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\karukan"
SectionEnd
