# 1) Let release-plz bump, commit and tag
release-plz update --allow-dirty

git add CHANGELOG.md
git add Cargo.lock
git add Cargo.toml
git commit --amend --no-edit

# 2) Grab the 5-char SHA of that new HEAD
$sha = git rev-parse --short=7 HEAD

# 3) Pull the version out of Cargo.toml
$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' |
            ForEach-Object { $_.Matches[0].Groups[1].Value })

# 4) Format today’s date as yy/MM/dd
$date = Get-Date -Format 'yy/MM/dd'



# List of crate directories
$crates = @("cardinal-uxn", "cardinal-varvara", "cardinal-gui", "cardinal-cli")

# Prepare LAST_RELEASE content with a global header
$lastReleaseContent = "# Multi-crate LAST_RELEASE`n"

foreach ($crate in $crates) {
    $changelogPath = Join-Path $crate "CHANGELOG.md"
    if (-not (Test-Path $changelogPath)) {
        Write-Warning "No CHANGELOG.md found for $crate, skipping."
        continue
    }

    # Read and parse CHANGELOG.md for this crate
    $lines = Get-Content -Path $changelogPath

    # Find first version section
    $startIndex = $null
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^## \[\d+\.\d+\.\d+\]') {
            $startIndex = $i
            break
        }
    }
    if ($startIndex -eq $null) {
        Write-Warning "Could not find version section in $changelogPath, skipping."
        continue
    }

    # Find end of the section
    $endIndex = $lines.Count
    for ($i = $startIndex + 1; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^## \[\d+\.\d+\.\d+\]') {
            $endIndex = $i
            break
        }
    }
    $changelogBodyLines = $lines[$startIndex..($endIndex - 1)]
    $changelogBody = $changelogBodyLines -join "`n"

    # Extract date
    if ($lines[$startIndex] -match '- (\d{4})-(\d{2})-(\d{2})') {
        $year = $matches[1].Substring(2)
        $month = $matches[2]
        $day = $matches[3]
        $date = "$year/$month/$day"
    } else {
        $date = Get-Date -Format 'yy/MM/dd'
    }

    # Get version from Cargo.toml in this crate
    $cargoPath = Join-Path $crate "Cargo.toml"
    $version = (Select-String -Path $cargoPath -Pattern '^version\s*=\s*"([^"]+)"' |
                ForEach-Object { $_.Matches[0].Groups[1].Value })

    # Compose first line for this crate
    $firstLine = "$date|$sha|$version"

    # Add crate section to LAST_RELEASE
    $lastReleaseContent += "`n## $crate`n$firstLine`n$changelogBody`n"
}

# Write LAST_RELEASE file
[System.IO.File]::WriteAllText("LAST_RELEASE", $lastReleaseContent, [System.Text.UTF8Encoding]::new($true))

Write-Host "Wrote multi-crate LAST_RELEASE"

git add LAST_RELEASE

Write-Host "Amended last commit to include LAST_RELEASE"
$existing = git log -1 --pretty=%B
$combined = "$existing`n`n# Multi-crate LAST_RELEASE"

$commitMsgFile = "COMMIT_MSG.tmp"
Set-Content -Path $commitMsgFile -Value $combined -Encoding UTF8
git commit --amend -F $commitMsgFile
Remove-Item $commitMsgFile
