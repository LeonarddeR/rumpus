; Rumpus Windows installer, compiled by `cargo xtask dist`, which passes:
;   /DVERSION=<display>  /DVERSIONNUM=<x.y.z.0>  /DSTAGING=<dir>  /DOUTDIR=<dir>  /DICONFILE=<rumpus.ico>
; STAGING contains windows\x86_64\rumpus.exe with the Windows MIDI Services runtime files next to it.

#ifndef VERSION
	#define VERSION "0.0.0"
#endif
#ifndef VERSIONNUM
	#define VERSIONNUM "0.0.0.0"
#endif
#ifndef STAGING
	#error STAGING is required (cargo xtask dist passes /DSTAGING=<dir>)
#endif
#ifndef OUTDIR
	#define OUTDIR "."
#endif
#define SRCDIR STAGING + "\windows\x86_64"

[Setup]
	AppId={{01BFEDEA-8480-464B-8717-B747D0EE988E}
	AppName=Rumpus
	AppVersion={#VERSION}
	AppPublisher=Leonard de Ruijter
	AppPublisherURL=https://github.com/LeonarddeR/rumpus
	AppSupportURL=https://github.com/LeonarddeR/rumpus/issues
	AppUpdatesURL=https://github.com/LeonarddeR/rumpus/releases
	DefaultDirName={autopf}\Rumpus
	PrivilegesRequired=admin
	PrivilegesRequiredOverridesAllowed=dialog
	DisableProgramGroupPage=yes
	DisableDirPage=no
	DisableWelcomePage=no
	LicenseFile=..\..\LICENSE
	OutputDir={#OUTDIR}
	OutputBaseFilename=rumpus_setup-x64
	Compression=lzma2
	SolidCompression=yes
	WizardStyle=modern
	UninstallDisplayIcon={app}\rumpus.exe
	ArchitecturesAllowed=x64compatible
	ArchitecturesInstallIn64BitMode=x64compatible
	VersionInfoVersion={#VERSIONNUM}
	ChangesAssociations=yes
#ifdef ICONFILE
	SetupIconFile={#ICONFILE}
#endif

[Languages]
	Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
	StartMenuShortcut=Start Menu shortcut
	DesktopShortcut=Desktop shortcut
	AssocMidi=Open MIDI files (*.mid, *.midi) with Rumpus by default
	MidiFileProgId=MIDI file
	LaunchApp=Launch Rumpus
	LegacyMode=This PC is set to the legacy MIDI API mode, so Rumpus cannot use Windows MIDI Services.%n%nSwitch the API mode back and restart Windows, then run this setup again. Instructions:%n%1
	NotAvailable=Windows MIDI Services is not available on this PC, so Rumpus cannot be installed.%n%nWindows MIDI Services is part of Windows 11 version 24H2 and later with all updates applied.
	ProbeFailed=Setup could not check whether Windows MIDI Services is available: %1

[Files]
	Source: "{#SRCDIR}\rumpus.exe"; DestDir: "{app}"; Flags: ignoreversion
	Source: "{#SRCDIR}\Windows.Devices.Midi2.dll"; DestDir: "{app}"; Flags: ignoreversion
	Source: "{#SRCDIR}\Windows.Devices.Midi2.pri"; DestDir: "{app}"; Flags: ignoreversion

[Tasks]
	Name: "startmenuicon"; Description: "{cm:StartMenuShortcut}"
	Name: "desktopicon"; Description: "{cm:DesktopShortcut}"; Flags: unchecked
	Name: "assoc_midi"; Description: "{cm:AssocMidi}"

[Icons]
	Name: "{autoprograms}\Rumpus"; Filename: "{app}\rumpus.exe"; Tasks: startmenuicon
	Name: "{autodesktop}\Rumpus"; Filename: "{app}\rumpus.exe"; Tasks: desktopicon

[Registry]
	; HKA is HKLM in administrative install mode and HKCU otherwise.
	Root: HKA; Subkey: "Software\Classes\Rumpus.MidiFile"; ValueType: string; ValueName: ""; ValueData: "{cm:MidiFileProgId}"; Flags: uninsdeletekey
	Root: HKA; Subkey: "Software\Classes\Rumpus.MidiFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\rumpus.exe,0"
	Root: HKA; Subkey: "Software\Classes\Rumpus.MidiFile\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\rumpus.exe"" ""%1"""
	Root: HKA; Subkey: "Software\Classes\.mid"; ValueType: string; ValueName: ""; ValueData: "Rumpus.MidiFile"; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_midi
	Root: HKA; Subkey: "Software\Classes\.mid\OpenWithProgids"; ValueType: string; ValueName: "Rumpus.MidiFile"; ValueData: ""; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_midi
	Root: HKA; Subkey: "Software\Classes\.midi"; ValueType: string; ValueName: ""; ValueData: "Rumpus.MidiFile"; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_midi
	Root: HKA; Subkey: "Software\Classes\.midi\OpenWithProgids"; ValueType: string; ValueName: "Rumpus.MidiFile"; ValueData: ""; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_midi

[Code]
const
	ApiModeHelpUrl = 'https://microsoft.github.io/MIDI/kb/how-to-change-api-mode/';
	// Exit codes of `rumpus.exe --probe`, defined in crates/rumpus/src/midi/backend.rs.
	ProbeOk = 0;
	ProbeLegacyMode = 1;

// Runs the app's own Windows MIDI Services check from {tmp}; it demand-starts the MIDI service,
// which can take a few seconds.
function InitializeSetup: Boolean;
var
	ResultCode: Integer;
	Message: String;
begin
	ExtractTemporaryFile('rumpus.exe');
	ExtractTemporaryFile('Windows.Devices.Midi2.dll');
	ExtractTemporaryFile('Windows.Devices.Midi2.pri');
	if not Exec(ExpandConstant('{tmp}\rumpus.exe'), '--probe', ExpandConstant('{tmp}'), SW_HIDE, ewWaitUntilTerminated, ResultCode) then
	begin
		SuppressibleMsgBox(FmtMessage(CustomMessage('ProbeFailed'), [SysErrorMessage(ResultCode)]), mbCriticalError, MB_OK, IDOK);
		Result := False;
		Exit;
	end;
	case ResultCode of
		ProbeOk: Message := '';
		ProbeLegacyMode: Message := FmtMessage(CustomMessage('LegacyMode'), [ApiModeHelpUrl]);
	else
		Message := CustomMessage('NotAvailable');
	end;
	Result := Message = '';
	if not Result then
		SuppressibleMsgBox(Message, mbCriticalError, MB_OK, IDOK);
end;

[Run]
	Filename: "{app}\rumpus.exe"; Description: "{cm:LaunchApp}"; Flags: nowait postinstall skipifsilent
