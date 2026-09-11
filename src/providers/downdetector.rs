use crate::core::monitor::HealthState;
use anyhow::{Context, anyhow};

pub fn check(
    client: &reqwest::blocking::Client,
    company_id: u64,
    token: &str,
) -> anyhow::Result<(HealthState, String)> {
    if company_id == 0 {
        return Err(anyhow!("company_id Downdetector invalide"));
    }

    if token.trim().is_empty() {
        return Err(anyhow!(
            "Token Downdetector absent. Définis DOWNDETECTOR_TOKEN ou saisis-le dans le panneau."
        ));
    }

    let url = format!("https://downdetectorapi.com/v2/companies/{company_id}/status");

    let response = client
        .get(url)
        .bearer_auth(token.trim())
        .send()
        .context("échec de la requête Downdetector")?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(anyhow!("Downdetector a répondu HTTP {status}"));
    }

    let raw = response
        .json::<String>()
        .context("réponse Downdetector inattendue")?;

    let result = match raw.as_str() {
        "success" => (HealthState::Operational, "Aucun problème détecté".into()),
        "warning" => (HealthState::Degraded, "Problèmes possibles détectés".into()),
        "danger" => (
            HealthState::Outage,
            "Incident détecté par Downdetector".into(),
        ),
        other => (
            HealthState::Unknown,
            format!("État Downdetector inconnu: {other}"),
        ),
    };

    Ok(result)
}
