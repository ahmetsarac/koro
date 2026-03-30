# Sıra önemli; db/migrations/*.sql dosyalarını ada göre sıralayıp uygular.
# Çalıştırma (repo kökünden): powershell -ExecutionPolicy Bypass -File .\scripts\apply-migrations.ps1
# Local compose: $env:POSTGRES_CONTAINER = "koro-postgres-local"; .\scripts\apply-migrations.ps1

$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Container = if ($env:POSTGRES_CONTAINER) { $env:POSTGRES_CONTAINER } else { "koro-postgres" }

$MigrationsDir = Join-Path $Root "db\migrations"
if (-not (Test-Path -LiteralPath $MigrationsDir)) {
    Write-Error "Klasör yok: $MigrationsDir"
}

$files = Get-ChildItem -LiteralPath $MigrationsDir -Filter "*.sql" | Sort-Object Name
if ($files.Count -eq 0) {
    Write-Error "db/migrations altında .sql yok."
}

foreach ($f in $files) {
    Write-Host "==> $($f.Name)"
    Get-Content -LiteralPath $f.FullName -Encoding UTF8 | docker exec -i $Container psql -U koro -d koro
    if ($LASTEXITCODE -ne 0) {
        Write-Error "psql hata: $($f.Name) (exit $LASTEXITCODE)"
    }
}

Write-Host "Tamam. ($($files.Count) dosya.)"
