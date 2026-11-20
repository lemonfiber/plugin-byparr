//! The manifest, as the core at the pinned commit reads it.

fn manifest() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/plugin.toml")).unwrap_or_default()
}

#[test]
fn the_pinned_core_refuses_nothing_in_the_manifest() {
    let read: Result<lemonfiber_plugin::Manifest, _> = toml::from_str(&manifest());
    let refused: Vec<String> = match read {
        Ok(manifest) => lemonfiber_plugin::refusals(&manifest, &[])
            .into_iter()
            .map(|violation| format!("{}: {}", violation.location, violation.message))
            .collect(),
        Err(why) => vec![why.to_string()],
    };
    assert!(refused.is_empty(), "{refused:#?}");
}

fn unfenced_proxies(manifest: &str) -> Vec<String> {
    toml::from_str::<lemonfiber_plugin::Manifest>(manifest).map_or_else(
        |why| vec![why.to_string()],
        |manifest| {
            let fetching: Vec<&str> = manifest
                .services
                .iter()
                .filter(|service| {
                    service.provides.iter().any(|capability| {
                        capability == lemonfiber_contract::capabilities::indexer::proxy::CAPABILITY
                    })
                })
                .map(|service| service.fronts.as_deref().unwrap_or(&service.id))
                .collect();
            manifest
                .services
                .iter()
                .filter(|service| fetching.contains(&service.id.as_str()))
                .filter(|service| {
                    !service
                        .network
                        .as_ref()
                        .is_some_and(|network| network.fenced)
                })
                .map(|service| service.id.clone())
                .collect()
        },
    )
}

#[test]
fn the_service_that_fetches_for_indexers_runs_fenced() {
    assert_eq!(unfenced_proxies(&manifest()), Vec::<String>::new());
}

#[test]
fn a_manifest_whose_indexer_proxy_service_is_not_fenced_is_refused() {
    let unfenced = manifest().replace("network     = { fenced = true }\n", "");
    assert_ne!(unfenced, manifest());
    assert_eq!(unfenced_proxies(&unfenced), vec!["byparr".to_owned()]);
}

fn published(manifest: &str) -> Vec<String> {
    toml::from_str::<lemonfiber_plugin::Manifest>(manifest).map_or_else(
        |why| vec![why.to_string()],
        |manifest| {
            manifest
                .services
                .iter()
                .filter(|service| service.port.is_some() || service.bind.is_some())
                .map(|service| service.id.clone())
                .collect()
        },
    )
}

#[test]
fn no_service_publishes_a_port_on_the_host() {
    assert_eq!(published(&manifest()), Vec::<String>::new());
}

#[test]
fn a_manifest_publishing_the_solver_on_the_host_is_refused() {
    let exposed = manifest().replace(
        "listens     = 8191\n",
        "port        = 8192\nbind        = \"loopback\"\nlistens     = 8191\n",
    );
    assert_ne!(exposed, manifest());
    assert_eq!(published(&exposed), vec!["byparr".to_owned()]);
}

fn solver_env(manifest: &str) -> Option<toml::Table> {
    let read: toml::Table = toml::from_str(manifest).ok()?;
    read.get("service")?
        .as_array()?
        .iter()
        .find(|service| service.get("id").and_then(toml::Value::as_str) == Some("byparr"))?
        .get("env")?
        .as_table()
        .cloned()
}

#[test]
fn the_browser_takes_the_house_s_zone_and_language_and_no_geoip_database() {
    let env = solver_env(&manifest()).unwrap_or_default();
    let setting = |name: &str| {
        env.get(name)
            .and_then(|value| value.get("setting"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    assert_eq!(setting("BROWSER_TIMEZONE").as_deref(), Some("timezone"));
    assert_eq!(setting("BROWSER_LOCALE").as_deref(), Some("locale"));
    assert_eq!(
        env.get("STEALTHFOX_GEOIP_MMDB")
            .and_then(toml::Value::as_str),
        Some("/nonexistent/geoip.mmdb")
    );
}
