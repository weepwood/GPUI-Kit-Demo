#define AppName "GPUI Kit Demo"
#define AppVersion "0.1.0"
#define AppExeName "gpui-kit-demo.exe"

[Setup]
SourceDir=..
AppId=GPUI-Kit-Demo-Weepwood
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=weepwood
DefaultDirName={autopf}\GPUI Kit Demo
DefaultGroupName={#AppName}
OutputDir=dist
OutputBaseFilename=GPUI-Kit-Demo-Setup-Windows-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
UninstallDisplayIcon={app}\{#AppExeName}

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "target\release\{#AppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent
