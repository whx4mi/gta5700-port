$base = 'F:\GTAVI-Build\gta5700'
$map = [ordered]@{
  'F:\GTAVI-Build\gta5700\repos\xcl' = "$base\repos\xcl"
  'F:\GTAVI-Build\gta5700\build\xcl-target' = "$base\build\xcl-target"
  'F:\GTAVI-Build\gta5700\repos\victory-fork' = "$base\repos\victory-fork"
  'F:\GTAVI-Build\gta5700\cache\xcl-dxil' = "$base\cache\xcl-dxil"
  'F:\GTAVI-Build\gta5700\tools' = "$base\tools"
  'F:\GTAVI-Build\gta5700\refs\' = "$base\refs\"
  'F:\GTAVI-Build\gta5700\backups\' = "$base\backups\"
  'F:\GTAVI-Build\gta5700\runs\victory-cache' = "$base\runs\victory-cache"
  'F:\GTAVI-Build\gta5700\runs\victory-capture' = "$base\runs\victory-capture"
  'F:\GTAVI-Build\gta5700\runs\victory-data' = "$base\runs\victory-data"
  'F:\GTAVI-Build\gta5700\runs\XCL-' = "$base\runs\XCL-"
}

$files = @()
$files += Get-ChildItem $base -Filter '*.md' -File -ErrorAction SilentlyContinue
$files += Get-ChildItem "$base\docs" -Filter '*.md' -File -ErrorAction SilentlyContinue
$files += Get-ChildItem "$base\tools" -Filter '*.ps1' -File -ErrorAction SilentlyContinue
$files += Get-ChildItem "$base\skills" -Filter '*.md' -File -Recurse -ErrorAction SilentlyContinue
$files += Get-ChildItem "$env:USERPROFILE\.config\opencode\skills\xcl-dev" -Filter '*.md' -File -Recurse -ErrorAction SilentlyContinue
$files += Get-ChildItem "$env:USERPROFILE\.config\opencode\skills\victory-fork-dev" -Filter '*.md' -File -Recurse -ErrorAction SilentlyContinue
$files += Get-Item "$base\repos\victory-fork\victory.toml" -ErrorAction SilentlyContinue

$n = 0
foreach ($f in $files) {
  if (-not $f) { continue }
  $text = Get-Content $f.FullName -Raw -ErrorAction SilentlyContinue
  if (-not $text) { continue }
  $orig = $text
  foreach ($k in $map.Keys) { $text = $text.Replace($k, $map[$k]) }
  if ($text -ne $orig) { Set-Content $f.FullName -Value $text -NoNewline -Encoding utf8; $n++ }
}
"$n arquivos atualizados"
