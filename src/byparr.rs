//! Byparr, asked through the FlareSolverr-compatible API it serves: a page fetched past its
//! challenge by the headless browser it drives, and whether it answers.
//!
//! Every address is screened before the browser is handed it: `http` or `https`, with no
//! user or password, naming its host by a name rather than a number, and a name every
//! address of which `lemonfiber_outward` passes. What is handed over is the address as
//! the screen read it. The screen is a first refusal, not the protection: the browser
//! follows redirects and resolves the name again, and what holds it off every private
//! address and the host's own is the fence Byparr runs inside.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use lemonfiber_adapter_kit::endpoint::{describe, Endpoint};
use lemonfiber_outward::{literal, outward, Inward};
use lemonfiber_ports::http::{Http, Method, Request, Response, JSON};
use lemonfiber_ports::resolve::Resolver;
use lemonfiber_ports::service::{Cookie, Failure, Solved, Solving};
use serde::de::IgnoredAny;
use serde::Deserialize;
use url::Url;

/// What every failure is blamed on.
pub const SERVICE: &str = "Byparr";

/// Where Byparr is asked to fetch a page.
const SOLVE: &str = "/v1";

/// Where Byparr describes the API it serves, which it answers once it has started.
const DESCRIBED: &str = "/openapi.json";

/// The command that fetches a page.
const GET: &str = "request.get";

/// What Byparr says of a page it fetched.
const SOLVED: &str = "ok";

/// The header a request's body type is named in.
const CONTENT_TYPE: &str = "Content-Type";

/// The status Byparr answers with where the page or its challenge outlasted its timeout.
const TIMED_OUT: u16 = 408;

/// The status Byparr answers with where its browser could not reach the site.
const UNREACHED: u16 = 502;

/// How long Byparr is given to fetch a page, inside the contract's deadline.
pub const SOLVE_WITHIN: Duration = Duration::from_secs(50);

/// How long the adapter waits for Byparr's answer, a little past what it gives it.
pub const ANSWER_WITHIN: Duration = Duration::from_secs(55);

/// The schemes a page is fetched over.
const FETCHED_OVER: [&str; 2] = ["http", "https"];

/// One Byparr, asked over its API at its address, handed only addresses the screen passes
/// as `resolver` says what a name stands for.
pub struct Byparr {
    endpoint: Endpoint,
    resolver: Arc<dyn Resolver>,
}

#[derive(Deserialize)]
struct Answered {
    status: String,
    #[serde(default)]
    solution: Option<Solution>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Solution {
    url: String,
    status: u16,
    #[serde(default)]
    cookies: Vec<Kept>,
    user_agent: String,
    response: String,
}

#[derive(Deserialize)]
struct Kept {
    name: String,
    value: String,
    domain: String,
}

#[derive(Deserialize)]
struct Described {
    paths: BTreeMap<String, Operations>,
}

#[derive(Deserialize)]
struct Operations {
    #[serde(default)]
    post: Option<IgnoredAny>,
}

impl Byparr {
    /// The Byparr at `address`, asked over `http`, every page's host resolved by `resolver`.
    pub fn new(
        http: Arc<dyn Http>,
        address: impl Into<String>,
        resolver: Arc<dyn Resolver>,
    ) -> Self {
        Self {
            endpoint: Endpoint::new(http, address, SERVICE),
            resolver,
        }
    }

    /// `url` as the screen read it, where it passes as an address out on the internet, or
    /// why it does not, naming the reason and never the address.
    async fn outside(&self, url: &str) -> Result<String, &'static str> {
        let read = Url::parse(url).map_err(|_| "the address cannot be read")?;
        if !FETCHED_OVER.contains(&read.scheme()) {
            return Err("only an http or https address is fetched");
        }
        if !read.username().is_empty() || read.password().is_some() {
            return Err("an address carrying a user or a password is not fetched");
        }
        let host = read.host_str().unwrap_or_default();
        let port = read.port_or_known_default().unwrap_or_default();
        if literal(host).is_some() {
            return Err("an address naming its host by number is not fetched");
        }
        outward(self.resolver.as_ref(), host, port)
            .await
            .map_err(|why| match why {
                Inward::Unresolved(_) => "the host could not be resolved",
                Inward::Nameless | Inward::Nowhere => "the host stands for no address",
                Inward::Internal(_) => {
                    "the host stands for this machine or a network of its own, not the internet"
                }
            })?;
        Ok(read.into())
    }

    /// Why Byparr did not hand back a page, in words that never carry the address: its
    /// timeout and its browser failing to reach the site named for what they are, every
    /// other status as the kit reads it.
    fn unfetched(&self, response: &Response) -> Failure {
        match response.status {
            TIMED_OUT => self
                .endpoint
                .refused("the page or its challenge outlasted the time Byparr was given"),
            UNREACHED => self
                .endpoint
                .refused("Byparr's browser could not reach the site"),
            _ => self.endpoint.refusal(response),
        }
    }
}

#[async_trait]
impl Solving for Byparr {
    async fn solve(&self, url: &str) -> Result<Solved, Failure> {
        let url = self.outside(url).await.map_err(|why| Failure::Disallowed {
            service: SERVICE.to_owned(),
            detail: why.to_owned(),
        })?;
        let body = serde_json::json!({
            "cmd": GET,
            "url": url,
            "maxTimeout": SOLVE_WITHIN.as_millis(),
        })
        .to_string();
        let response = self
            .endpoint
            .send(&Request {
                method: Method::Post,
                url: self.endpoint.url(SOLVE),
                headers: vec![(CONTENT_TYPE.to_owned(), JSON.to_owned())],
                body: Some(body),
                pinned: None,
            })
            .await?;
        if !response.is_success() {
            return Err(self.unfetched(&response));
        }
        let answered: Answered = self.endpoint.decode(&response, "the page fetched")?;
        match answered.solution {
            Some(solution) if answered.status == SOLVED => Ok(Solved {
                status: solution.status,
                url: solution.url,
                user_agent: solution.user_agent,
                cookies: solution
                    .cookies
                    .into_iter()
                    .map(|kept| Cookie {
                        name: kept.name,
                        value: kept.value,
                        domain: kept.domain,
                    })
                    .collect(),
                page: solution.response,
            }),
            _ => Err(self.endpoint.refused(&describe(&response))),
        }
    }

    async fn answers(&self) -> Result<bool, Failure> {
        let response = self
            .endpoint
            .send(&Request {
                method: Method::Get,
                url: self.endpoint.url(DESCRIBED),
                headers: Vec::new(),
                body: None,
                pinned: None,
            })
            .await?;
        let described: Described = self.endpoint.decode(&response, "the API it describes")?;
        Ok(described
            .paths
            .get(SOLVE)
            .is_some_and(|operations| operations.post.is_some()))
    }
}

#[cfg(test)]
mod tests;
