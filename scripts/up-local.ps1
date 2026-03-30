# docker-compose.local.yml sırayı kendi çözüyor; bu dosya yalnızca kısayol.
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root
docker compose -f docker-compose.local.yml up --build -d
