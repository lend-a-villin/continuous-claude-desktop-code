# CCDC tray app: shows an icon in the notification area while it runs.
#   - watches the Claude app log itself (cheap) and starts Node only when there
#     is work: `ccdc.mjs signout` on sign-out, `ccdc.mjs signin` after sign-in.
#     No Node process stays in memory between account switches.
#   - menu: auto sync on/off, move sessions now, delete copies after sign-in,
#     copy/delete speed with a time estimate, Language, log, account names, quit
# Start it with ccdc-tray.vbs (no console window). UI text comes from locales.csv.

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
[System.Windows.Forms.Application]::EnableVisualStyles()
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$script = Join-Path $here 'ccdc.mjs'
$settingsFile = Join-Path $here 'ccdc.json'
$stateDir = Join-Path $env:LOCALAPPDATA 'CCDC'
$logFile = Join-Path $stateDir 'watch.log'
New-Item -ItemType Directory -Force $stateDir | Out-Null
$appLog = if ($env:CCDC_APP_LOG) { $env:CCDC_APP_LOG } else { Join-Path $env:LOCALAPPDATA 'Claude\Logs\main.log' }
$settleSeconds = 6  # the app reads its session folder twice within ~2 s of sign-in

# ---------- settings (ccdc.json) ----------
. (Join-Path $here 'i18n.ps1')

function Read-Settings {
    if (-not (Test-Path $settingsFile)) { return $null }
    try { return ([System.IO.File]::ReadAllText($settingsFile) | ConvertFrom-Json) } catch { return $null }
}

# sets one key in ccdc.json, keeping everything else (account names included)
function Save-Setting([string]$name, $value) {
    $settings = Read-Settings
    if ((Test-Path $settingsFile) -and -not $settings) {
        Show-Balloon (T 'settings_unreadable') 'Warning'   # never overwrite an unreadable file
        return $false
    }
    if (-not $settings) { $settings = [pscustomobject]@{ aliases = [pscustomobject]@{} } }
    $settings | Add-Member -NotePropertyName $name -NotePropertyValue $value -Force
    [System.IO.File]::WriteAllText($settingsFile, ($settings | ConvertTo-Json -Depth 5), (New-Object System.Text.UTF8Encoding($false)))
    return $true
}

$langOverride = (Read-Settings).language
Import-CcdcStrings $here $langOverride

$nodeCmd = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeCmd) {
    [System.Windows.Forms.MessageBox]::Show((T 'node_missing'), 'CCDC') | Out-Null
    exit 1
}
$node = $nodeCmd.Source

# one tray icon per Windows user
$created = $false
$mutex = New-Object System.Threading.Mutex($true, 'Local\CCDCTray', [ref]$created)
if (-not $created) {
    [System.Windows.Forms.MessageBox]::Show((T 'already_running'), 'CCDC') | Out-Null
    exit 0
}

# ---------- memory: give unused pages back while idle ----------
if (-not ('CcdcMem.Native' -as [type])) {
    Add-Type -Namespace CcdcMem -Name Native -MemberDefinition '[DllImport("psapi.dll")] public static extern bool EmptyWorkingSet(System.IntPtr hProcess);'
}
function Reduce-Memory {
    [System.GC]::Collect()
    [System.GC]::WaitForPendingFinalizers()
    try { [void][CcdcMem.Native]::EmptyWorkingSet([System.Diagnostics.Process]::GetCurrentProcess().Handle) } catch {}
}

# ---------- icon: orange circle with a white "C" ----------
function New-TrayIcon([System.Drawing.Color]$color) {
    $bmp = New-Object System.Drawing.Bitmap 32, 32
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.TextRenderingHint = 'AntiAliasGridFit'
    $g.FillEllipse((New-Object System.Drawing.SolidBrush $color), 1, 1, 30, 30)
    $font = New-Object System.Drawing.Font('Segoe UI', 15, [System.Drawing.FontStyle]::Bold)
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = 'Center'; $sf.LineAlignment = 'Center'
    $g.DrawString('C', $font, [System.Drawing.Brushes]::White, (New-Object System.Drawing.RectangleF(0, 1, 32, 32)), $sf)
    $g.Dispose()
    [System.Drawing.Icon]::FromHandle($bmp.GetHicon())
}
$iconOn = New-TrayIcon ([System.Drawing.Color]::FromArgb(217, 119, 87))
$iconOff = New-TrayIcon ([System.Drawing.Color]::FromArgb(140, 140, 140))

$tray = New-Object System.Windows.Forms.NotifyIcon
$tray.Visible = $true

# ---------- helpers ----------
function Invoke-CCDC([string[]]$argList) {
    $out = & $node $script @argList 2>&1 | Out-String
    [pscustomobject]@{ Ok = ($LASTEXITCODE -eq 0); Text = $out.Trim() }
}

function Show-Balloon([string]$text, [string]$kind = 'Info') {
    $tray.BalloonTipTitle = 'CCDC'
    $tray.BalloonTipText = $text
    $tray.BalloonTipIcon = $kind
    $tray.ShowBalloonTip(4000)
}

function Get-ShortId([string]$id) { $id.Substring(0, 8) }

function Write-WatchLog([string]$msg) {
    Add-Content -Path $logFile -Value "[$(Get-Date -Format T)] $msg" -Encoding UTF8
}

# the notification-area tooltip may hold at most 63 characters
function Set-Tooltip([string]$text) {
    if ($text.Length -gt 63) { $text = $text.Substring(0, 62) + [char]0x2026 }
    $tray.Text = $text
}

# watch.log line (always English) -> balloon text in the UI language
function Format-Event([string]$msg) {
    if ($msg -match 'copying sessions into every account folder \((\d+) file\(s\), (\d+MB)(?:, about (\d+)s)?\)') {
        $text = T 'ev_copy_start' $Matches[1] $Matches[2]
        if ($Matches[3]) { $text += ' ' + (T 'eta_seconds' $Matches[3]) }
        return $text
    }
    if ($msg -match '^Sign-out detected') { return T 'ev_signout_nothing' }
    if ($msg -match 'Copy done: .*\((\d+) file\(s\), (\d+MB), ([\d.]+)s\)') {
        return T 'ev_copy_done' $Matches[1] $Matches[3]
    }
    if ($msg -match 'Sign-in detected: (\w+)/(\w+), copies kept') { return T 'ev_signin_keep' $Matches[1] }
    if ($msg -match 'Sign-in detected: (\w+)/(\w+), removing copies in (\d+)s') {
        return T 'ev_signin' $Matches[1] $Matches[3]
    }
    if ($msg -match 'kept sessions in (\w+)/(\w+); removed (\d+) copies') {
        return T 'ev_cleanup' $Matches[3] $Matches[1]
    }
    if ($msg -match '^Note: (\d+) session') { return T 'ev_late_move' $Matches[1] }
    # "replicate failed: …", "copy failed <file>: …": the error itself stays as Windows wrote it
    if ($msg -match 'failed(?: [^:]+)?:\s*(.*)$') { return T 'ev_failed' $Matches[1] }
    return $msg
}

# output of `ccdc.mjs move` -> balloon text in the UI language
function Format-MoveResult([string]$text, [bool]$ok, [string]$accountName) {
    if (-not $ok) {
        if ($text -match 'has no session folder yet') { return T 'move_no_folder' $accountName }
        $err = ($text -split "`r?`n" | Where-Object { $_ } | Select-Object -Last 1) -replace '^Error:\s*', ''
        return T 'move_failed' $err
    }
    if ($text -match 'Nothing to move') { return T 'move_nothing' $accountName }
    $m = [regex]::Match($text, 'Moved (\d+) file\(s\), removed (\d+) copies(?:, replaced (\d+))?')
    if ($m.Success) {
        $replaced = if ($m.Groups[3].Success) { $m.Groups[3].Value } else { '0' }
        return T 'move_done' $accountName $m.Groups[1].Value $m.Groups[2].Value $replaced
    }
    return ($text -split "`r?`n" | Select-Object -Last 1)
}

# ---------- jobs: Node runs only while there is work ----------
$script:jobQueue = New-Object System.Collections.ArrayList
$script:job = $null

function Add-Job([string[]]$jobArgs) {
    # a newer sign-out/sign-in makes a queued one of the same kind pointless
    for ($i = $script:jobQueue.Count - 1; $i -ge 0; $i--) {
        if ($script:jobQueue[$i][0] -eq $jobArgs[0]) { $script:jobQueue.RemoveAt($i) }
    }
    [void]$script:jobQueue.Add($jobArgs)
}

function Step-Jobs {
    if ($script:job) {
        if (-not $script:job.HasExited) { return }
        $script:job.Dispose()
        $script:job = $null
        Reduce-Memory
    }
    if ($script:jobQueue.Count -eq 0) { return }
    $next = $script:jobQueue[0]
    $script:jobQueue.RemoveAt(0)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $node
    $quoted = ($next | ForEach-Object { if ($_ -match '\s') { "`"$_`"" } else { $_ } }) -join ' '
    $psi.Arguments = "--max-old-space-size=64 --expose-gc `"$script`" $quoted --log `"$logFile`""
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $script:job = [System.Diagnostics.Process]::Start($psi)
}

# ---------- app log follower: detects sign-out and sign-in ----------
$script:autoSync = $true
$script:appLogPos = if (Test-Path $appLog) { (Get-Item $appLog).Length } else { 0 }
$script:appLogCarry = ''
$script:signedIn = ''        # "account/org" last seen
$script:signinDue = $null    # when to run the sign-in job
$script:signinArgs = $null
$logoutRe = [regex]'Login-state transition \(loggedOut: false \S+ true'
$initRe = [regex]'\[LocalSessionManager\] Initialization succeeded \S+ accountId=([0-9a-fA-F-]{36}), orgId=([0-9a-fA-F-]{36})'

$script:appLogBuf = New-Object byte[] 65536   # reused for every read
function Read-AppLog {
    $info = New-Object System.IO.FileInfo $appLog
    if (-not $info.Exists) { return }
    $len = $info.Length
    if ($len -lt $script:appLogPos) { $script:appLogPos = 0 }   # rotated
    if ($len -eq $script:appLogPos) { return }
    $fs = [System.IO.File]::Open($appLog, 'Open', 'Read', 'ReadWrite')
    try {
        [void]$fs.Seek($script:appLogPos, 'Begin')
        $n = $fs.Read($script:appLogBuf, 0, [int][Math]::Min($script:appLogBuf.Length, $len - $script:appLogPos))
        $script:appLogPos += $n
    } finally { $fs.Dispose() }
    if ($n -le 0) { return }
    $chunk = [System.Text.Encoding]::UTF8.GetString($script:appLogBuf, 0, $n)
    # most new lines are unrelated; skip the split unless a line we care about could be there
    if ($script:appLogCarry -eq '' -and $chunk.IndexOf('Login-state transition') -lt 0 -and $chunk.IndexOf('[LocalSessionManager]') -lt 0) {
        $cut = $chunk.LastIndexOf("`n")
        $script:appLogCarry = if ($cut -ge 0) { $chunk.Substring($cut + 1) } else { $chunk }
        return
    }
    $lines = ($script:appLogCarry + $chunk) -split "`r?`n"
    $script:appLogCarry = $lines[-1]
    if ($script:appLogCarry.Length -gt 16384) { $script:appLogCarry = '' }
    for ($i = 0; $i -lt $lines.Count - 1; $i++) {
        $line = $lines[$i]
        if ($logoutRe.IsMatch($line)) {
            $script:signedIn = ''
            $script:signinDue = $null
            Add-Job @('signout')
            continue
        }
        $m = $initRe.Match($line)
        if ($m.Success) {
            $acc = $m.Groups[1].Value.ToLower(); $org = $m.Groups[2].Value.ToLower()
            if ($script:signedIn -ne "$acc/$org") {
                $script:signedIn = "$acc/$org"
                $keep = (Read-Settings).keepCopies -ne $false
                $what = if ($keep) { 'copies kept' } else { "removing copies in $($settleSeconds)s" }
                Write-WatchLog "Sign-in detected: $($acc.Substring(0,8))/$($org.Substring(0,8)), $what"
            }
            $script:signinDue = (Get-Date).AddSeconds($settleSeconds)
            $script:signinArgs = @('signin', '--account', $acc, '--org', $org)
        }
    }
}

function Update-State {
    $on = $script:autoSync
    $tray.Icon = if ($on) { $iconOn } else { $iconOff }
    Set-Tooltip $(if ($on) { T 'tooltip_on' } else { T 'tooltip_off' })
    $miWatch.Checked = $on
    $miHeader.Text = if ($on) { T 'header_on' } else { T 'header_off' }
}

# ---------- menu ----------
$menu = New-Object System.Windows.Forms.ContextMenuStrip

$miHeader = New-Object System.Windows.Forms.ToolStripMenuItem
$miHeader.Enabled = $false
[void]$menu.Items.Add($miHeader)
[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

$miWatch = New-Object System.Windows.Forms.ToolStripMenuItem
$miWatch.add_Click({
    $script:autoSync = -not $script:autoSync
    if ($script:autoSync) {
        # catch up with whatever happened while it was off
        $script:appLogPos = if (Test-Path $appLog) { (Get-Item $appLog).Length } else { 0 }
        Add-Job @('startup')
    } else { $script:signinDue = $null }
    Update-State
})
[void]$menu.Items.Add($miWatch)

$miMoveTo = New-Object System.Windows.Forms.ToolStripMenuItem
[void]$miMoveTo.DropDownItems.Add('...')
[void]$menu.Items.Add($miMoveTo)

[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

# delete copies after sign-in (on) or keep them for fewer SSD writes (off)
$miDelete = New-Object System.Windows.Forms.ToolStripMenuItem
$miDelete.add_Click({
    $keep = (Read-Settings).keepCopies -ne $false
    [void](Save-Setting 'keepCopies' (-not $keep))
})
[void]$menu.Items.Add($miDelete)

$miSpeed = New-Object System.Windows.Forms.ToolStripMenuItem
foreach ($s in 'fast', 'normal', 'gentle') {
    $item = New-Object System.Windows.Forms.ToolStripMenuItem
    $item.Tag = $s
    $item.add_Click({ param($sender) [void](Save-Setting 'speed' $sender.Tag) })
    [void]$miSpeed.DropDownItems.Add($item)
}
[void]$menu.Items.Add($miSpeed)

$miEstimate = New-Object System.Windows.Forms.ToolStripMenuItem
$miEstimate.Enabled = $false
[void]$menu.Items.Add($miEstimate)

[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

# "Language" is always written in English, and each language in its own name,
# so someone who cannot read the current language can still find their own.
$miLang = New-Object System.Windows.Forms.ToolStripMenuItem 'Language'
[void]$menu.Items.Add($miLang)

function Save-Language([string]$code) {
    [void](Save-Setting 'language' $code)
    $script:langOverride = $code
    Import-CcdcStrings $here $code
    Set-MenuTexts
}

function Build-LanguageMenu {
    $miLang.DropDownItems.Clear()
    $auto = New-Object System.Windows.Forms.ToolStripMenuItem 'Auto (Windows display language)'
    $auto.Checked = [string]::IsNullOrEmpty($script:langOverride)
    $auto.add_Click({ Save-Language '' })
    [void]$miLang.DropDownItems.Add($auto)
    [void]$miLang.DropDownItems.Add((New-Object System.Windows.Forms.ToolStripSeparator))
    foreach ($code in $script:CcdcLanguages.Keys) {
        $item = New-Object System.Windows.Forms.ToolStripMenuItem $script:CcdcLanguages[$code]
        $item.Tag = $code
        $item.Checked = (-not [string]::IsNullOrEmpty($script:langOverride)) -and ($code -eq $script:CcdcLang)
        $item.add_Click({ param($sender) Save-Language $sender.Tag })
        [void]$miLang.DropDownItems.Add($item)
    }
}

[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

$miLog = New-Object System.Windows.Forms.ToolStripMenuItem
$miLog.add_Click({
    if (-not (Test-Path $logFile)) { New-Item -ItemType File $logFile | Out-Null }
    Start-Process notepad.exe -ArgumentList "`"$logFile`""
})
[void]$menu.Items.Add($miLog)

$miAlias = New-Object System.Windows.Forms.ToolStripMenuItem
$miAlias.add_Click({
    $settings = Read-Settings
    if (-not $settings -or -not $settings.aliases -or @($settings.aliases.PSObject.Properties).Count -eq 0) {
        $r = Invoke-CCDC @('accounts')
        $aliases = [ordered]@{}
        if ($r.Ok) { foreach ($a in ($r.Text | ConvertFrom-Json)) { $aliases[(Get-ShortId $a.account)] = '' } }
        [void](Save-Setting 'aliases' ([pscustomobject]$aliases))
    }
    Start-Process notepad.exe -ArgumentList "`"$settingsFile`""
})
[void]$menu.Items.Add($miAlias)

[void]$menu.Items.Add((New-Object System.Windows.Forms.ToolStripSeparator))

$miQuit = New-Object System.Windows.Forms.ToolStripMenuItem
$miQuit.add_Click({
    $timer.Stop()
    if ($script:job -and -not $script:job.HasExited) { $script:job.WaitForExit(30000) | Out-Null }  # let a copy finish
    $tray.Visible = $false
    $tray.Dispose()
    [System.Windows.Forms.Application]::Exit()
})
[void]$menu.Items.Add($miQuit)

# (re)labels everything in the current language; "Language" itself stays English
function Set-MenuTexts {
    $menu.RightToLeft = $(if ($script:CcdcLang -in 'ar', 'he', 'fa', 'ur') { 'Yes' } else { 'No' })
    $miWatch.Text = T 'menu_auto'
    $miMoveTo.Text = T 'menu_move'
    $miDelete.Text = T 'menu_delete_copies'
    $miSpeed.Text = T 'menu_speed'
    foreach ($item in $miSpeed.DropDownItems) { $item.Text = T "speed_$($item.Tag)" }
    $miLog.Text = T 'menu_log'
    $miAlias.Text = T 'menu_alias'
    $miQuit.Text = T 'menu_quit'
    Build-LanguageMenu
    Update-State
}
Set-MenuTexts

# accounts, settings and the time estimate are read fresh every time the menu opens
$menu.add_Opening({
    $settings = Read-Settings
    $keep = $settings.keepCopies -ne $false
    $speed = if ($settings.speed -in 'fast', 'normal', 'gentle') { $settings.speed } else { 'fast' }
    $miDelete.Checked = -not $keep
    foreach ($item in $miSpeed.DropDownItems) { $item.Checked = ($item.Tag -eq $speed) }

    $est = Invoke-CCDC @('estimate', '--json')
    if ($est.Ok) {
        $e = $est.Text | ConvertFrom-Json
        $miEstimate.Text = if ($e.keepCopies) { T 'est_keep' $e.copySeconds } else { T 'est_delete' $e.copySeconds $e.cleanupSeconds }
        $miEstimate.Visible = $true
    } else { $miEstimate.Visible = $false }

    $miMoveTo.DropDownItems.Clear()
    $r = Invoke-CCDC @('accounts')
    if (-not $r.Ok) {
        $item = $miMoveTo.DropDownItems.Add((T 'accounts_error'))
        $item.Enabled = $false
        return
    }
    foreach ($a in ($r.Text | ConvertFrom-Json)) {
        $name = if ($a.alias) { "$($a.alias) ($(Get-ShortId $a.account))" } else { Get-ShortId $a.account }
        $label = T 'account_label' $name $a.sessions
        if ($a.signedIn) { $label += '  ' + (T 'signed_in_tag') }
        $item = New-Object System.Windows.Forms.ToolStripMenuItem $label
        $item.Tag = $a
        $item.add_Click({
            param($sender)
            $acc = $sender.Tag
            if (-not $acc.signedIn) {
                $ans = [System.Windows.Forms.MessageBox]::Show((T 'move_confirm'), 'CCDC', 'YesNo', 'Warning')
                if ($ans -ne 'Yes') { return }
            }
            $res = Invoke-CCDC @('move', '--to', $acc.account)
            $accName = if ($acc.alias) { "$($acc.alias) ($(Get-ShortId $acc.account))" } else { Get-ShortId $acc.account }
            Show-Balloon (Format-MoveResult $res.Text $res.Ok $accName) ($(if ($res.Ok) { 'Info' } else { 'Error' }))
            $last = ($res.Text -split "`r?`n" | Select-Object -Last 1)
            Write-WatchLog "manual: $last"
        })
        [void]$miMoveTo.DropDownItems.Add($item)
    }
})
$menu.add_Closed({ Reduce-Memory })

$tray.ContextMenuStrip = $menu
$tray.add_MouseClick({
    param($s, $e)
    if ($e.Button -eq 'Left') {
        # left click opens the same menu
        $m = [System.Windows.Forms.NotifyIcon].GetMethod('ShowContextMenu', [System.Reflection.BindingFlags]'Instance,NonPublic')
        $m.Invoke($tray, $null)
    }
})

# ---------- one timer: app log, jobs, balloons, memory ----------
$script:logPos = if (Test-Path $logFile) { (Get-Item $logFile).Length } else { 0 }
$script:ticks = 0
$timer = New-Object System.Windows.Forms.Timer
$timer.Interval = 500
$timer.add_Tick({
    $script:ticks++
    if ($script:autoSync) {
        Read-AppLog
        if ($script:signinDue -and (Get-Date) -ge $script:signinDue) {
            $script:signinDue = $null
            Add-Job $script:signinArgs
        }
    }
    Step-Jobs
    if ($script:ticks % 120 -eq 0 -and -not $script:job) { Reduce-Memory }   # every minute when idle

    # balloons for what the jobs wrote to watch.log
    $wl = New-Object System.IO.FileInfo $logFile
    if (-not $wl.Exists) { return }
    $len = $wl.Length
    if ($len -lt $script:logPos) { $script:logPos = 0 }
    if ($len -eq $script:logPos) { return }
    $fs = [System.IO.File]::Open($logFile, 'Open', 'Read', 'ReadWrite')
    try {
        [void]$fs.Seek($script:logPos, 'Begin')
        $text = (New-Object System.IO.StreamReader($fs, [System.Text.Encoding]::UTF8)).ReadToEnd()
        $script:logPos = $len
    } finally { $fs.Dispose() }
    foreach ($line in ($text -split "`r?`n")) {
        $msg = $line -replace '^\[[^\]]*\]\s*', ''
        if ($line -match 'LOW_DISK') {
            $m = [regex]::Match($line, 'needs (\d+MB), only (\d+MB)')
            $need = if ($m.Success) { $m.Groups[1].Value } else { '?' }
            $free = if ($m.Success) { $m.Groups[2].Value } else { '?' }
            $timer.Stop()
            [System.Windows.Forms.MessageBox]::Show((T 'lowdisk_body' $need $free), (T 'lowdisk_title'), 'OK', 'Warning') | Out-Null
            $timer.Start()
        } elseif ($line -match 'Sign-out detected|Sign-in detected|Copy done|kept sessions|failed|Note:' -and $line -notmatch 'manual:') {
            Show-Balloon (Format-Event $msg)
        }
    }
})
$timer.Start()

Add-Job @('startup')
Update-State
Show-Balloon (T 'started')
Reduce-Memory

$ctx = New-Object System.Windows.Forms.ApplicationContext
[System.Windows.Forms.Application]::Run($ctx)
$mutex.ReleaseMutex()
