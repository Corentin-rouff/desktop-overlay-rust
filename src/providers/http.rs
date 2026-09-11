use crate::core::monitor::HealthState;
use anyhow::Context;

pub fn check(
    client: &reqwest::blocking::Client,
    url: &str,
    expected_status: u16,
) -> anyhow::Result<(HealthState, String)> {
    let response = client
        .get(url)
        .send()
        .with_context(|| format!("échec HTTP vers {url}"))?;

    let actual = response.status().as_u16();
    if actual == expected_status {
        Ok((
            HealthState::Operational,
            format!("HTTP {actual} — service joignable"),
        ))
    } else if (500..=599).contains(&actual) {
        Ok((
            HealthState::Outage,
            format!("HTTP {actual} — erreur serveur"),
        ))
    } else {
        Ok((
            HealthState::Degraded,
            format!("HTTP {actual}, attendu {expected_status}"),
        ))
    }
}
