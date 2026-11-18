use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;
use lemonfiber_ports::resolve::Resolver;

use super::{Again, APART, TRIES};

const PUBLIC: IpAddr = IpAddr::V4(Ipv4Addr::new(93, 184, 215, 14));

/// A resolver that cannot answer its first `failing` asks and answers [`PUBLIC`] after.
struct Recovering {
    failing: u32,
    asked: AtomicU32,
}

impl Recovering {
    fn after(failing: u32) -> Self {
        Self {
            failing,
            asked: AtomicU32::new(0),
        }
    }

    fn asked(&self) -> u32 {
        self.asked.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Resolver for Recovering {
    async fn addresses(&self, _host: &str, _port: u16) -> Result<Vec<IpAddr>, String> {
        if self.asked.fetch_add(1, Ordering::SeqCst) < self.failing {
            Err("Temporary failure in name resolution".to_owned())
        } else {
            Ok(vec![PUBLIC])
        }
    }
}

#[tokio::test(start_paused = true)]
async fn a_name_the_resolver_could_not_answer_is_asked_again_until_it_answers() {
    let again = Again(Recovering::after(TRIES - 1));
    let started = tokio::time::Instant::now();

    let answered = again.addresses("example.com", 443).await;

    assert_eq!(answered, Ok(vec![PUBLIC]));
    assert_eq!(again.0.asked(), TRIES);
    assert_eq!(started.elapsed(), APART * (TRIES - 1));
}

#[tokio::test(start_paused = true)]
async fn a_name_answered_at_once_is_asked_once() {
    let again = Again(Recovering::after(0));
    let started = tokio::time::Instant::now();

    assert_eq!(again.addresses("example.com", 443).await, Ok(vec![PUBLIC]));
    assert_eq!(again.0.asked(), 1);
    assert_eq!(started.elapsed(), std::time::Duration::ZERO);
}

#[tokio::test(start_paused = true)]
async fn a_name_never_answered_is_unresolved_after_every_try() {
    let again = Again(Recovering::after(u32::MAX));

    let answered = again.addresses("example.com", 443).await;

    assert_eq!(
        answered,
        Err("Temporary failure in name resolution".to_owned())
    );
    assert_eq!(again.0.asked(), TRIES);
}
