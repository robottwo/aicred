[Setup]
AppName=GenAI Key Finder
AppVersion=0.1.0
DefaultDirName={pf}\GenAI Key Finder
DefaultGroupName=GenAI Key Finder
OutputDir=userdocs:Inno Setup Examples Output
OutputBaseFilename=genai-keyfinder-setup
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=admin

[Files]
Source: "target\x86_64-pc-windows-msvc\release\keyfinder.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\GenAI Key Finder"; Filename: "{app}\keyfinder.exe"
Name: "{commondesktop}\GenAI Key Finder"; Filename: "{app}\keyfinder.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\keyfinder.exe"; Description: "Launch GenAI Key Finder"; Flags: nowait postinstall skipifsilent

[Code]
procedure InitializeWizard;
begin
  // Custom initialization if needed
end;