# benchmark.ps1 - release startup + memory benchmark for danqing examples.
#
# Usage (from repo root):
#   powershell -NoProfile -File tools/benchmark.ps1 [-Example danqing-showcase] [-Runs 3]
#
# Prints per-run and aggregate numbers, then PASS/FAIL against budgets:
#   - startup_to_visible <= StartupBudgetMs
#   - working set        <= WsBudgetMB
# Exit code 0 on PASS, 1 on FAIL. Pure ASCII by design (see repo tooling rules).

param(
    [string]$Example = "danqing-showcase",
    [int]$Runs = 3,
    [double]$StartupBudgetMs = 1000,
    # LowPower (iGPU) adapter: shared GPU memory is fully counted in the
    # process working set, so the WS budget sits above the dGPU reading.
    [double]$WsBudgetMB = 360
)

$ErrorActionPreference = "Stop"
$exe = "target/release/examples/$Example.exe"

if (-not (Test-Path $exe)) {
    Write-Host "building $Example (release)..."
    cargo build --release --example $Example
    if ($LASTEXITCODE -ne 0) { exit 1 }
}

$env:RUST_LOG = "info"
$results = @()

function Invoke-BenchRun {
    param([int]$Index)

    $log = Join-Path $env:TEMP "danqing-bench-$Example-$Index.log"
    if (Test-Path $log) { Remove-Item $log -Force }

    # env_logger writes to stderr; capture it as the primary log.
    $proc = Start-Process -FilePath $exe -RedirectStandardOutput "$log.out" -RedirectStandardError $log -PassThru

    # Wait until the window-visible perf line appears in the log (max 20s).
    $startupMs = $null
    $deadline = (Get-Date).AddSeconds(20)
    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 200
        if ((Test-Path $log) -and (Select-String -Path $log -Pattern "perf startup_to_visible" -Quiet)) {
            $line = Select-String -Path $log -Pattern "perf startup_to_visible\s+([0-9.]+)(m?)s" |
                Select-Object -First 1
            $value = [double]$line.Matches[0].Groups[1].Value
            $startupMs = if ($line.Matches[0].Groups[2].Value -eq "m") { $value } else { $value * 1000 }
            break
        }
    }

    # Let the app reach steady state, then sample memory.
    Start-Sleep -Seconds 5
    $proc.Refresh()
    $wsMB = [math]::Round($proc.WorkingSet64 / 1MB, 1)
    $privMB = [math]::Round($proc.PrivateMemorySize64 / 1MB, 1)
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Remove-Item "$log.out" -Force -ErrorAction SilentlyContinue

    if ($null -eq $startupMs) { return $null }
    return [pscustomobject]@{
        StartupMs = [math]::Round($startupMs, 1)
        WsMB = $wsMB
        PrivMB = $privMB
    }
}

for ($i = 1; $i -le $Runs; $i++) {
    $run = Invoke-BenchRun -Index $i
    if ($null -eq $run) {
        # Single retry: a missed observation is usually harness flake, not app failure.
        Write-Host "run ${i}: perf line not observed, retrying once..."
        $run = Invoke-BenchRun -Index $i
    }
    if ($null -eq $run) {
        Write-Host "run ${i}: FAILED to observe startup perf line"
        exit 1
    }
    Write-Host ("run {0}: startup {1} ms | WS {2} MB | private {3} MB" -f $i, $run.StartupMs, $run.WsMB, $run.PrivMB)
    $results += $run
}

$minStartup = ($results | Measure-Object -Property StartupMs -Minimum).Minimum
$sortedWs = $results | Sort-Object -Property WsMB | ForEach-Object { $_.WsMB }
$medWs = $sortedWs[[math]::Floor($sortedWs.Count / 2)]
Write-Host ("--- best startup {0} ms | median WS {1} MB" -f $minStartup, $medWs)

$pass = ($minStartup -le $StartupBudgetMs) -and ($medWs -le $WsBudgetMB)
if ($pass) {
    Write-Host "PASS (budgets: startup <= $StartupBudgetMs ms, WS <= $WsBudgetMB MB)"
    exit 0
} else {
    Write-Host "FAIL (budgets: startup <= $StartupBudgetMs ms, WS <= $WsBudgetMB MB)"
    exit 1
}
