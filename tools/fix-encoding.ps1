# Repara mojibake causado por Get-Content/Set-Content com ANSI no PowerShell 5.1.
# Sem caracteres nao-ASCII no codigo: detecta pelos codigos Unicode.
$roots = @(
  'F:\GTAVI-Build\gta5700',
  "$env:USERPROFILE\.config\opencode\skills\xcl-dev",
  "$env:USERPROFILE\.config\opencode\skills\victory-fork-dev"
)
$a = [string][char]0x00C3
$b = [string][char]0x00E2
$files = @()
foreach ($r in $roots) {
  $files += Get-ChildItem $r -File -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.Extension -in '.md','.ps1','.toml' }
}
$n = 0
foreach ($f in $files) {
  $text = [IO.File]::ReadAllText($f.FullName, [Text.Encoding]::UTF8)
  if (-not ($text.Contains($a) -or $text.Contains($b))) { continue }
  for ($pass = 0; $pass -lt 4; $pass++) {
    $bytes = [Text.Encoding]::GetEncoding(1252).GetBytes($text)
    $fixed = [Text.Encoding]::UTF8.GetString($bytes)
    if ($fixed -eq $text) { break }
    $text = $fixed
    if (-not ($text.Contains($a) -or $text.Contains($b))) { break }
  }
  [IO.File]::WriteAllText($f.FullName, $text, [Text.Encoding]::UTF8)
  $n++
}
"$n arquivos reparados"
