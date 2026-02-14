# oura-cli

Unofficial CLI for [Oura Ring](https://ouraring.com) — sleep, readiness, and activity scores from your terminal.

## Setup

1. Get a Personal Access Token from [Oura Cloud](https://cloud.ouraring.com/personal-access-tokens)
2. Export it: `export OURA_TOKEN=your_token_here`
3. Install: `cargo install --path .`

## Usage

```
oura                          # sleep + readiness + activity scores (today)
oura scores [DATE]            # same as above, with optional date
oura sleep [DATE]             # detailed sleep breakdown
oura readiness [DATE]         # readiness score + contributors
oura activity [DATE]          # steps, calories, movement
oura hrv [DATE]               # heart rate variability from sleep
oura stress [DATE]            # daily stress summary
oura trend [-d DAYS]          # score trend over last N days (default: 7)
oura json <ENDPOINT> [DATE]   # raw JSON for any API endpoint
```

`DATE` accepts `YYYY-MM-DD`, `today`, or `yesterday`. Defaults to today.

## Example

```
$ oura
  Sleep 82  Readiness 79  Activity 91

$ oura readiness
  Readiness Score: 79
  Temp Deviation:  +0.2°C
  Activity Balance        85
  Body Temperature        100
  HRV Balance             75
  ...
```

## Disclaimer

This project is not affiliated with, endorsed by, or connected to Oura Health Oy. "Oura" is a trademark of Oura Health Oy.
