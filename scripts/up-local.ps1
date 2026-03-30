# Tek komut: local compose — postgres → migrate → sqlx-prepare → up --build
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

$f = "docker-compose.local.yml"
docker compose -f $f up -d postgres
docker compose -f $f run --rm migrate
docker compose -f $f run --rm sqlx-prepare
docker compose -f $f up --build -d
