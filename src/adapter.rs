//! What this adapter says it is, and the kit that serves its contract.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::indexer;
use lemonfiber_ports::service::Solving as _;

use crate::byparr::Byparr;

/// Where Byparr answers, by its service id and the port it listens on.
pub const ADDRESS: &str = "http://byparr:8191";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "Byparr";

/// The Byparr release this adapter's recordings were taken from.
pub const RELEASE: &str = "3.1.1";

/// The digest of the image that release's recordings were taken from.
pub const DIGEST: &str = "sha256:c66fa161187b8fa527bfad7eefa2647ea1c553f77df68cbffd6fe83366d1787b";

/// What this adapter says it is: the contract it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![lemonfiber_contract::spoken(
            indexer::proxy::CAPABILITY,
            indexer::proxy::MAJOR,
        )],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contract, served from `solver` under `key`, ready while Byparr describes its API.
#[must_use]
pub fn kit(key: String, solver: Arc<Byparr>) -> Kit {
    let asked = Arc::clone(&solver);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await.unwrap_or(false) })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(indexer::proxy::served(solver))
}

#[cfg(test)]
mod tests;
