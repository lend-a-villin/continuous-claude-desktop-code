# UI text for the tray app and the installer, read from locales.csv.
#
# locales.csv is one sheet: one row per language, one column per message.
# To add a language, add one row whose "lang" cell is a Windows culture name
# ("fr", "pt-BR", "zh-Hant", ...). Write line breaks inside a cell as \n and
# keep {0}, {1} placeholders. Empty cells fall back to English.
# The "name" column is the language's own name, shown in the tray's Language menu.
#
# The language follows Windows' display language; "language" in ccdc.json overrides it.

function Import-CcdcStrings([string]$dir, [string]$override) {
    $table = @{}  # case-insensitive keys
    $script:CcdcLanguages = [ordered]@{}  # lang -> the language's own name, in sheet order
    foreach ($row in (Import-Csv -Path (Join-Path $dir 'locales.csv') -Encoding UTF8)) {
        if (-not $row.lang) { continue }
        $code = $row.lang.Trim()
        $table[$code] = $row
        $script:CcdcLanguages[$code] = $(if ($row.name) { $row.name } else { $code })
    }
    # Ask Windows for the user's display language. CurrentUICulture is not used:
    # console programs report English when the console code page is UTF-8 (chcp 65001).
    $ui = [System.Globalization.CultureInfo]::CurrentUICulture
    try {
        if (-not ('CcdcI18n.Native' -as [type])) {
            Add-Type -Namespace CcdcI18n -Name Native -MemberDefinition '[DllImport("kernel32.dll")] public static extern ushort GetUserDefaultUILanguage();'
        }
        $ui = New-Object System.Globalization.CultureInfo([int][CcdcI18n.Native]::GetUserDefaultUILanguage())
    } catch {}
    $candidates = @()
    if ($override) { $candidates += $override }
    $candidates += $ui.Name                       # e.g. pt-BR, zh-TW
    if ($ui.TwoLetterISOLanguageName -eq 'zh') {
        $candidates += $(if ($ui.Name -match 'TW|HK|MO|Hant') { 'zh-Hant' } else { 'zh-Hans' })
    }
    $candidates += $ui.TwoLetterISOLanguageName   # e.g. pt, zh
    $candidates += 'en'
    $script:CcdcLang = 'en'
    foreach ($c in $candidates) {
        if ($c -and $table.ContainsKey($c)) { $script:CcdcLang = $c; break }
    }
    $script:CcdcStrings = $table[$script:CcdcLang]
    $script:CcdcEnglish = $table['en']
}

# T 'key' [args...] -> translated text with {0}, {1} filled in
function T([string]$key) {
    $s = $null
    if ($script:CcdcStrings) { $s = $script:CcdcStrings.$key }
    if ([string]::IsNullOrEmpty($s) -and $script:CcdcEnglish) { $s = $script:CcdcEnglish.$key }
    if ([string]::IsNullOrEmpty($s)) { $s = $key }
    $s = $s.Replace('\n', "`n")
    if ($args.Count -gt 0) { return [string]::Format($s, [object[]]$args) }
    return $s
}
