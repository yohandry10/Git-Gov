# KAN-22 Workflow PowerShell Splatting Fix

Date: 2026-04-28

## Problem

`Risk Tier Baseline Calibration` scheduled run `24999681550` failed on 2026-04-27.

The workflow built a PowerShell array like `"-GitGovUrl", $env:GITGOV_URL, ...` and invoked:

```powershell
& ./scripts/control-plane/calibrate_risk_tier_baseline.ps1 @args
```

Array splatting passes values positionally. It does not convert string pairs into named parameters. The result was `-RepoFullName` being bound into the validated `Tier` parameter.

`Desktop Updater Readiness` had the same pattern and failed inside its optional job by binding `gitgov/src-tauri/tauri.conf.json` into `TimeoutSeconds`.

## Fix

Updated:

- `.github/workflows/risk-tier-baseline-calibration.yml`
- `.github/workflows/desktop-updater-readiness.yml`

Both workflows now use hashtable splatting:

```powershell
$scriptArgs = @{
  ParameterName = $value
}
& ./script.ps1 @scriptArgs
```

## Validation

- `.\scripts\control-plane\calibrate_risk_tier_baseline.ps1` with hashtable splatting generated a report successfully.
- Risk-tier validation result: readiness `92/100`, composite risk `8/100`, one SLA breach.
- `.\scripts\deploy\validate_desktop_updater_readiness.ps1` with hashtable splatting completed with expected optional `WARN` state while endpoint probing was skipped.
- Post-merge manual Risk Tier Baseline run `25049577630` confirmed the calibration step succeeded, then exposed a separate artifact upload issue because `report_path` was not visible to `actions/upload-artifact`.
- The workflow now writes `report_path` to `$GITHUB_OUTPUT` with `[System.IO.File]::AppendAllText(..., [System.Text.Encoding]::UTF8)`.
- `git diff --check` passed.
- `.\scripts\security\publication_guard.ps1` passed.

## Operating Note

Use hashtable splatting for named parameters in GitHub Actions PowerShell blocks. Avoid array splatting with `"-Param", value` pairs unless positional binding is explicitly intended.
