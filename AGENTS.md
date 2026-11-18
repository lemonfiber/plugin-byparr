# AGENTS.md — plugin-byparr

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

Byparr as a first-party lemonfiber plugin: `plugin.toml` and the adapter that
speaks the `indexer.proxy@1` contract in front of Byparr, the challenge solver
that stands in for FlareSolverr where `F14-R31` proves `indexer.proxy` by a
substitute. The adapter is the Rust crate at the root, built on
`lemonfiber-adapter-kit` from the core at the commit `Cargo.toml` pins, into the
image the `Dockerfile` builds. `release.yml` publishes that image from every
commit on `main` and from the train's tag. `plugin.toml` runs Byparr behind that
adapter, which provides `indexer.proxy`; lemonfiber asks Byparr through it alone,
and the indexer fetches through Byparr itself.

The adapter asks Byparr through the FlareSolverr-compatible API Byparr serves:
`POST /v1` with `request.get` to fetch a page, and `/openapi.json`, which Byparr
answers without fetching anything, to tell whether it answers.

## The rules you cannot break

- **The plugin id is the service's id** (`ARCH-R266`). The container, the
  configuration directory, the settings and the journal keep their names.
- **A substitute, not a move in place.** Byparr declares no `replaces`.
- **Byparr publishes no port on the host.** Only the adapter and the indexer reach
  it, on the stack's network, so nothing on the machine asks it for a page past
  the adapter's screen. The fence keeps every connection it makes, redirects
  included, off the machine, the home network and the stack's own services.
- **`native = "flaresolverr"` on Byparr.** Byparr serves the FlareSolverr API, so
  the core tells the indexer Byparr in that API, under the tag that sends an
  indexer site through it (`ARCH-R428`). lemonfiber's own calls reach the adapter,
  which speaks the contract (`ARCH-R253`).
- **Its proofs' recordings sit under `recordings/byparr/`**, and the adapter's
  conformance cases under `conformance/indexer.proxy@1/` (`ARCH-R271`).
- **Every service declares `reaches` and `asks_for`** (`ARCH-R270`).
- **The reader is spec's `first-party-plugin` workflow's,** and lemonfiber
  records the recordings itself. There is no Python here.
- **No `unwrap`, `expect`, `panic` or indexing** outside tests. The lints in
  `Cargo.toml` are the core's.
- **Byparr runs fenced, and the fence is the boundary.** Its service says
  `network = { fenced = true }`, so it runs inside the fence lemonfiber writes
  beside it, whose rules refuse every private address and the host's own.
  `tests/manifest.rs` refuses a manifest whose `indexer.proxy` service, or the
  service it fronts, is not fenced.
- **Byparr's page loader answers nobody.** Byparr's `POST /load` fetches pages as
  `POST /v1` does and answers anybody unless `OWUI_API_KEY` is set, so lemonfiber
  mints `loader-key` into it before Byparr first starts and hands it to nobody
  (`ARCH-R247`).
- **The adapter screens every address first.** It is `http` or `https`, carries
  no user or password, names its host by a name rather than a number, and every
  address that name resolves to passes `lemonfiber_outward`. A name the resolver
  cannot answer is asked again, as often and as far apart as `src/resolving.rs`
  holds, before it is refused as unresolved. Anything else is
  refused naming the reason, never the address, before Byparr is asked, and what
  is handed over is the address as the screen read it. The browser follows
  redirects and resolves names again, so the screen is a first refusal and never
  the protection.
- **No refusal carries the address.** Byparr's `408` and `502` carry a detail
  that can name it, so the adapter names each for what it is instead.
- **The adapter answers inside the contract's deadline.** Byparr is given
  `SOLVE_WITHIN` to fetch a page and the adapter waits `ANSWER_WITHIN` for it,
  both under the 60 seconds `indexer.proxy@1` allows.
- **The core's crates move with the train.** The `rev` in `Cargo.toml` is the
  core commit `targets.toml` names, the same for every `lemonfiber-*` crate.
- **Moving Byparr's digest** means recording every response again against the
  new image in the same change, moving `RELEASE` and `DIGEST` in
  `src/adapter.rs` with it, and committing the `proofs.json` the checks write.

## Checks

```
just ci
```

The gates that need nothing but this tree. The plugin verdict and the
recordings run in CI; `just recordings` takes the recordings again locally, with
Docker.
