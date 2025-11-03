[Setup]
AppName=AICred
AppVersion=0.1.0
DefaultDirName={pf}\AICred
DefaultGroupName=AICred
OutputDir=userdocs:Inno Setup Examples Output
OutputBaseFilename=aicred-setup
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=admin

[Files]
Source: "target\x86_64-pc-windows-msvc\release\aicred.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\AICred"; Filename: "{app}\aicred.exe"
Name: "{commondesktop}\AICred"; Filename: "{app}\aicred.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\aicred.exe"; Description: "Launch AICred"; Flags: nowait postinstall skipifsilent

[Code]
procedure InitializeWizard;
begin
  // Custom initialization if needed
end;