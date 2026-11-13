# Byparr for lemonfiber

**Gets past bot-protection on some indexers.**
Byparr is a challenge solver lemonfiber can run in place of FlareSolverr.
Without it: Those indexers return nothing.

## How lemonfiber carries it

This repository holds the adapter that speaks lemonfiber's `indexer.proxy@1`
contract in front of Byparr: a Rust crate built on the core's adapter kit, into
the image `ghcr.io/lemonfiber/byparr-adapter`, published from every commit on
`main` and from each tag the release train cuts. The adapter asks Byparr through
the FlareSolverr-compatible API Byparr serves. `plugin.toml` runs Byparr alone,
from its own image, `ghcr.io/thephaseless/byparr`, pinned by tag and digest.

## The fence

Byparr's browser fetches whatever address it is handed. Its service is declared
`network = { fenced = true }`, so lemonfiber runs it inside a fence whose rules
refuse every connection to a private address or to the host. That fence is the
boundary. The adapter also screens each address before handing it over,
refusing any that is not `http` or `https`, carries a user or password, names
its host by number, or resolves to an address of this machine or a network of
its own. The screen is a first refusal only, because the browser follows
redirects and resolves names again.

Byparr also serves an Open WebUI page loader at `POST /load`, which fetches
pages as `POST /v1` does. lemonfiber mints the key that loader answers to and
hands it to nobody, so the loader fetches nothing for anybody.

## Where Byparr differs from FlareSolverr

- **Its root redirects to its documentation**, and its `/health` sends its
  browser to an outside site. The adapter is ready while Byparr describes its
  API at `/openapi.json`, which it answers without fetching anything.
- **It answers `200` for every page it loads**, whatever status the site
  answered, so the status the adapter hands on is Byparr's.
- **A timeout is `408` and an unreachable site `502`**, each with a detail that
  can carry the address. The adapter names each for what it is and never passes
  the detail on.
- **Each time it starts it reaches outside the house**: it downloads a GeoIP
  database from GitHub and asks a lookup service for the house's address, so
  its browser's timezone and language match where the house appears to be.
  Without the internet it starts in UTC and `en-US`.

## Settings

| Setting | Default | What it decides |
| --- | --- | --- |
| `byparr:log-level` | `info` | How much Byparr writes to its log |

## What is here

| Path | What it is |
| --- | --- |
| `plugin.toml` | The plugin's manifest |
| `Cargo.toml`, `src/` | The adapter, built on `lemonfiber-adapter-kit` |
| `Dockerfile` | The adapter's image |
| `recordings/byparr/` | The responses its proofs are held to |
| `targets.toml` | The lemonfiber its checks run against |
| `proofs.json` | What those checks said, committed |
| `status.toml` | The requirements this repository holds its part of |

## Checks

The plugin verdict is one job of spec's
[`first-party-plugin`](https://github.com/lemonfiber/spec/blob/main/.github/workflows/first-party-plugin.yml)
workflow: the manifest as the lemonfiber `targets.toml` names reads it, every
proof against its recording, and every pinned digest. `recordings` in
`plugin.yml` takes every recording again with the adapter built from this tree.
`rust.yml` holds the adapter to its format, lints, tests, every line covered and
its dependency policy. `just ci` runs the gates that need nothing but this tree.

## Licence

The data in this repository is under the licence in `LICENSE`.
Byparr itself is under GPL-3.0 and is not distributed here: this repository
names an image, it does not contain one.
