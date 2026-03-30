# Varsayılan docker-compose: postgres → migrate → sqlx-prepare → up --build
$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root
docker compose up -d postgres
docker compose run --rm migrate
docker compose run --rm sqlx-prepare
docker compose up --build -d
