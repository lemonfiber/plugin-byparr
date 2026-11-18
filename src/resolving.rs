//! A resolver whose every name it could not answer is asked again.

use std::net::IpAddr;
use std::time::Duration;

use async_trait::async_trait;
use lemonfiber_ports::resolve::Resolver;

/// How many times a name is asked before it is taken as unresolved.
pub const TRIES: u32 = 3;

/// How long apart two asks of one name are.
pub const APART: Duration = Duration::from_secs(1);

/// The resolver it holds, a name it could not answer asked again up to [`TRIES`] times in
/// all, [`APART`] apart.
pub struct Again<R>(pub R);

#[async_trait]
impl<R: Resolver> Resolver for Again<R> {
    async fn addresses(&self, host: &str, port: u16) -> Result<Vec<IpAddr>, String> {
        let mut asked = 1;
        loop {
            match self.0.addresses(host, port).await {
                Err(_) if asked < TRIES => {
                    asked += 1;
                    tokio::time::sleep(APART).await;
                }
                answered => return answered,
            }
        }
    }
}

#[cfg(test)]
mod tests;
