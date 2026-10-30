Unicode true
Name "Horizon Optimized"
OutFile "${OUTPUT_FILE}"
InstallDir "$EXEDIR\Horizon Optimized"
RequestExecutionLevel user
SetCompressor /SOLID lzma
SetCompressorDictSize 32
Icon "${PROJECT_DIR}/packaging/icons/horizon.ico"
UninstallIcon "${PROJECT_DIR}/packaging/icons/horizon.ico"
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"
!include "WinMessages.nsh"
!include "x64.nsh"
!define REGKEY "Software\HorizonOptimized"
!define UNINSTALLKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\HorizonOptimized"
!define MUI_WELCOMEPAGE_TITLE "Instalar o Horizon Optimized"
!define MUI_WELCOMEPAGE_TEXT "Escolha em qual disco guardar o Horizon, seus mundos, versões, downloads e arquivos temporários.$\r$\n$\r$\nVocê pode escolher uma pasta no D:, E: ou outro disco. Não é necessário instalar em C:.$\r$\n$\r$\nO login Microsoft usa o launcher oficial, sem modificações. É necessária uma conta que possui Minecraft Java."
!insertmacro MUI_PAGE_WELCOME
!define MUI_DIRECTORYPAGE_TEXT_TOP "Escolha uma pasta com espaço livre. Os arquivos grandes do Horizon serão guardados dentro dela. A escolha também vale para outros atalhos do Horizon deste usuário."
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\Horizon-Optimized.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Abrir o Horizon Optimized"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "PortugueseBR"

Function .onInit
  SetShellVarContext current
  SetRegView 64
  ${IfNot} ${RunningX64}
    MessageBox MB_ICONSTOP "Este programa precisa do Windows de 64 bits."
    Abort
  ${EndIf}
  ReadRegStr $0 HKCU "${REGKEY}" "InstallDir"
  ${If} $0 != ""
    StrCpy $INSTDIR $0
  ${EndIf}
FunctionEnd

Function .onVerifyInstDir
  ${GetRoot} "$INSTDIR" $0
  ${If} $INSTDIR == $0
    Abort
  ${EndIf}
FunctionEnd

; Persist only Horizon's two names, so instance shortcuts also use the chosen disk.
; Save previous values once and restore them on uninstall, if still ours.
!macro SaveHome NAME SUFFIX
  ReadRegStr $0 HKCU "${REGKEY}" "Saved-${NAME}"
  ${If} $0 != "1"
    ClearErrors
    ReadRegStr $1 HKCU "Environment" "${NAME}"
    ${If} ${Errors}
      WriteRegStr HKCU "${REGKEY}" "Had-${NAME}" "0"
    ${Else}
      WriteRegStr HKCU "${REGKEY}" "Had-${NAME}" "1"
      WriteRegStr HKCU "${REGKEY}" "Old-${NAME}" "$1"
    ${EndIf}
    WriteRegStr HKCU "${REGKEY}" "Saved-${NAME}" "1"
  ${EndIf}
  WriteRegStr HKCU "Environment" "${NAME}" "$INSTDIR\data\${SUFFIX}"
!macroend

Section "Horizon Optimized"
  SetOutPath "$INSTDIR"
  SetOverwrite on
  ClearErrors
  File /r "${PAYLOAD_DIR}/*"
  IfErrors failed
  ; Never replace the user's settings during an upgrade/reinstallation.
  IfFileExists "$INSTDIR\data\launcher\settings.json" settings_done
  SetOutPath "$INSTDIR\data\launcher"
  File /oname=settings.json "${DEFAULT_SETTINGS}"
  settings_done:
  FileOpen $0 "$INSTDIR\data\launcher\versions\v0.2.98\.installed" w
  IfErrors failed
  FileClose $0
  !insertmacro SaveHome "HORIZON_LAUNCHER_HOME" "launcher"
  !insertmacro SaveHome "HORIZON_HOME" "game"
  WriteRegStr HKCU "${REGKEY}" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\Desinstalar.exe"
  CreateShortcut "$DESKTOP\Horizon Optimized.lnk" "$INSTDIR\Horizon-Optimized.exe"
  CreateDirectory "$SMPROGRAMS\Horizon Optimized"
  CreateShortcut "$SMPROGRAMS\Horizon Optimized\Horizon Optimized.lnk" "$INSTDIR\Horizon-Optimized.exe"
  WriteRegStr HKCU "${UNINSTALLKEY}" "DisplayName" "Horizon Optimized"
  WriteRegStr HKCU "${UNINSTALLKEY}" "UninstallString" '$\"$INSTDIR\Desinstalar.exe$\"'
  WriteRegStr HKCU "${UNINSTALLKEY}" "DisplayIcon" "$INSTDIR\Horizon-Optimized.exe"
  WriteRegStr HKCU "${UNINSTALLKEY}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKCU "${UNINSTALLKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALLKEY}" "NoRepair" 1
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
  Goto done
  failed:
    MessageBox MB_ICONSTOP "Não foi possível concluir a instalação. Verifique o espaço e o acesso ao disco escolhido."
    Abort
  done:
SectionEnd

!macro RestoreHome NAME SUFFIX
  ReadRegStr $0 HKCU "Environment" "${NAME}"
  ${If} $0 == "$INSTDIR\data\${SUFFIX}"
    ReadRegStr $1 HKCU "${REGKEY}" "Had-${NAME}"
    ${If} $1 == "1"
      ReadRegStr $2 HKCU "${REGKEY}" "Old-${NAME}"
      WriteRegStr HKCU "Environment" "${NAME}" "$2"
    ${Else}
      DeleteRegValue HKCU "Environment" "${NAME}"
    ${EndIf}
  ${EndIf}
!macroend

Section "Uninstall"
  SetShellVarContext current
  SetRegView 64
  ReadRegStr $3 HKCU "${REGKEY}" "InstallDir"
  ${If} $3 == $INSTDIR
    !insertmacro RestoreHome "HORIZON_LAUNCHER_HOME" "launcher"
    !insertmacro RestoreHome "HORIZON_HOME" "game"
    Delete "$DESKTOP\Horizon Optimized.lnk"
    RMDir /r "$SMPROGRAMS\Horizon Optimized"
    DeleteRegKey HKCU "${UNINSTALLKEY}"
    DeleteRegKey HKCU "${REGKEY}"
  ${EndIf}
  RMDir /r "$INSTDIR\runtime"
  Delete "$INSTDIR\Horizon-Optimized.exe"
  Delete "$INSTDIR\Desinstalar.exe"
  ; Worlds, settings, accounts and game data stay for the user to keep or remove.
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd
