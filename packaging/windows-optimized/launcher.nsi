; Small front door for the unmodified, Microsoft-configured Horizon launcher.
Unicode true
Name "Horizon Optimized"
OutFile "${OUTPUT_FILE}"
RequestExecutionLevel user
SilentInstall silent
AutoCloseWindow true
Icon "${PROJECT_DIR}/packaging/icons/horizon.ico"
!include "FileFunc.nsh"

Section
  SetOutPath "$EXEDIR"
  IfFileExists "$EXEDIR\data\launcher\versions\v0.2.98\.installed" +3
    MessageBox MB_ICONSTOP "A instalação não foi concluída. Execute o instalador novamente."
    Quit
  IfFileExists "$EXEDIR\data\launcher\versions\v0.2.98\horizon-client.exe" +3
    MessageBox MB_ICONSTOP "Instalação incompleta. Execute o instalador do Horizon Optimized novamente."
    Quit
  CreateDirectory "$EXEDIR\data\temp"
  IfErrors 0 +3
    MessageBox MB_ICONSTOP "Não foi possível gravar nesta pasta. Verifique o acesso ao disco escolhido."
    Quit
  ; Only this process and its children: never change the system's TEMP/Cargo folders.
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_LAUNCHER_HOME", w "$EXEDIR\data\launcher") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_HOME", w "$EXEDIR\data\game") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "TEMP", w "$EXEDIR\data\temp") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "TMP", w "$EXEDIR\data\temp") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "TMPDIR", w "$EXEDIR\data\temp") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "CARGO_HOME", w "$EXEDIR\data\toolchain\cargo") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "RUSTUP_HOME", w "$EXEDIR\data\toolchain\rustup") i .r0'
  ; Preserve the packaged edition instead of silently replacing it with upstream.
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_LAUNCHER_NO_SELF_UPDATE", w "1") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_LAUNCHER_MOTION", w "off") i .r0'
  System::Call 'kernel32::SetEnvironmentVariableW(w "HORIZON_MENU_FPS", w "30") i .r0'
  ${GetParameters} $R1
  ClearErrors
  Exec '"$EXEDIR\runtime\horizon-launcher.exe" $R1'
  IfErrors 0 +2
    MessageBox MB_ICONSTOP "O launcher não iniciou. Reinstale o Horizon Optimized nesta pasta."
SectionEnd
