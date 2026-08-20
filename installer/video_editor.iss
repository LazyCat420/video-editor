; Inno Setup Script for LazyCat420 Video Editor
; Download Inno Setup from: https://jrsoftware.org/isdl.php
; Compile with: iscc installer\video_editor.iss

#define MyAppName "Video Editor"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "LazyCat420"
#define MyAppExeName "video-editor.exe"

[Setup]
AppId={{E5D9F438-7B62-4C91-95EA-3A1BCF745420}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=VideoEditorSetup-v{#MyAppVersion}
SetupIconFile=..\assets\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
; Main Executable
Source: "..\target\x86_64-pc-windows-gnullvm\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Application Assets
Source: "..\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
; FFmpeg engine. The app shells out to ffmpeg/ffprobe to import, preview and export, so
; without these it installs but cannot open a single video. find_ffmpeg_executable() looks
; in <exe>\bin before falling back to PATH. Run scripts/package-release.sh first to
; populate this directory.
Source: "..\dist\VideoEditor-Portable\bin\*"; DestDir: "{app}\bin"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
