; Native Windows controls: no GPUI, GPU scene or animated background.
Unicode true
Name "Horizon Optimized - Leve"
Caption "Horizon Optimized - Intel HD"
OutFile "${OUTPUT_FILE}"
RequestExecutionLevel user
Icon "${PROJECT_DIR}/packaging/icons/horizon.ico"
!include "nsDialogs.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"
Page custom LauncherPage
Var Dialog
Var OpenGL
Var Button

Function .onInit
  SetOutPath "$EXEDIR"
  CreateDirectory "$EXEDIR\data\temp"
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_LAUNCHER_HOME", w "$EXEDIR\data\launcher") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_HOME", w "$EXEDIR\data\game") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "TEMP", w "$EXEDIR\data\temp") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "TMP", w "$EXEDIR\data\temp") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "TMPDIR", w "$EXEDIR\data\temp") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "CARGO_HOME", w "$EXEDIR\data\toolchain\cargo") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "RUSTUP_HOME", w "$EXEDIR\data\toolchain\rustup") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_LAUNCHER_NO_SELF_UPDATE", w "1") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_LAUNCHER_MOTION", w "off") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_MENU_FPS", w "30") i .r0'
FunctionEnd

Function LauncherPage
  GetDlgItem $0 $HWNDPARENT 1
  ShowWindow $0 ${SW_HIDE}
  GetDlgItem $0 $HWNDPARENT 2
  SendMessage $0 ${WM_SETTEXT} 0 "STR:Sair"
  nsDialogs::Create 1018
  Pop $Dialog
  ${NSD_CreateLabel} 0 0 100% 30u "Modo leve para Intel HD. Login e preparo usam o launcher oficial, sem abrir sua interface 3D. Todos os recursos avançados continuam em Launcher completo."
  Pop $0
  ${NSD_CreateButton} 0 37u 48% 22u "1. Entrar com Microsoft"
  Pop $Button
  ${NSD_OnClick} $Button SignIn
  ${NSD_CreateButton} 52% 37u 48% 22u "2. Jogar"
  Pop $Button
  ${NSD_OnClick} $Button Play
  ${NSD_CreateCheckbox} 0 65u 100% 12u "OpenGL para Intel HD 2500/4000 (recomendado)"
  Pop $OpenGL
  ${NSD_SetState} $OpenGL ${BST_CHECKED}
  ${NSD_CreateButton} 0 87u 48% 20u "Aplicar modo leve"
  Pop $Button
  ${NSD_OnClick} $Button Apply
  ${NSD_CreateButton} 52% 87u 48% 20u "Restaurar backup original"
  Pop $Button
  ${NSD_OnClick} $Button Restore
  ${NSD_CreateButton} 0 113u 48% 20u "Launcher completo"
  Pop $Button
  ${NSD_OnClick} $Button Full
  ${NSD_CreateButton} 52% 113u 48% 20u "Abrir pasta dos dados"
  Pop $Button
  ${NSD_OnClick} $Button Folder
  nsDialogs::Show
FunctionEnd

Function SignIn
  MessageBox MB_OK "Uma janela mostrará o endereço da Microsoft e um código. Confirme o login no navegador. Depois de aparecer Signed in, feche essa janela e clique em Jogar."
  Exec '"$SYSDIR\cmd.exe" /D /S /K ""$EXEDIR\runtime\horizon-launcher.exe" account sign-in"'
FunctionEnd

Function Play
  IfFileExists "$EXEDIR\data\launcher\versions\v0.2.98\.installed" +3
    MessageBox MB_ICONSTOP "Instalação incompleta. Reinstale na mesma pasta."
    Return
  ${NSD_GetState} $OpenGL $0
  ${If} $0 == ${BST_CHECKED}
    System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_GRAPHICS_API", w "opengl") i .r0'
  ${Else}
    System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_GRAPHICS_API", w "auto") i .r0'
  ${EndIf}
  ClearErrors
  Exec '"$SYSDIR\cmd.exe" /D /S /K ""$EXEDIR\runtime\horizon-launcher.exe" play --version v0.2.98 --arg=--width --arg=960 --arg=--height --arg=540"'
  IfErrors 0 +3
    MessageBox MB_ICONSTOP "Não foi possível iniciar. Abra o launcher completo para verificar o erro."
    Return
  Quit
FunctionEnd

Function Apply
  ExecWait '"$EXEDIR\runtime\horizon-light-profile.exe" apply "$EXEDIR"' $0
  ${If} $0 == 0
    MessageBox MB_OK "Modo leve aplicado. Suas opções anteriores têm backup. Reinicie o jogo para usar a configuração."
  ${Else}
    MessageBox MB_ICONSTOP "Não foi possível aplicar. Verifique o acesso à pasta."
  ${EndIf}
FunctionEnd
Function Restore
  ExecWait '"$EXEDIR\runtime\horizon-light-profile.exe" restore "$EXEDIR"' $0
  ${If} $0 == 0
    MessageBox MB_OK "Backup restaurado. As configurações gráficas foram recuperadas; mundos e contas foram preservados."
  ${Else}
    MessageBox MB_ICONSTOP "Não há backup disponível ou a pasta não pode ser gravada."
  ${EndIf}
FunctionEnd
Function Full
  Exec '"$EXEDIR\Horizon-Completo.exe"'
FunctionEnd
Function Folder
  ExecShell "open" "$EXEDIR\data"
FunctionEnd
Section
SectionEnd
