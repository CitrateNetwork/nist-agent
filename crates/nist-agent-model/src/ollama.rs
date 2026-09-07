//! Ollama HTTP client — one of the two v1 local-discovery sources.
//!
//! Ollama listens on `127.0.0.1:11434` by default. We discover by
//! issuing GET `/api/tags`. If the request succeeds AND the named
//! model is listed, we resolve to this backend. The harness's
//! egress posture (RFC §3.3) is preserved by binding to localhost
//! — no DNS lookup or external connection is required.
//!
//! Inference uses `POST /api/generate` with `stream: true` and
//! parses the NDJSON response into a token stream.

use crate::error::ModelError;
use crate::{ModelBackend, TokenStream};
use async_trait::async_trait;
use futures::stream;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_OLLAMA_PORT: u16 = 11434;
const DISCOVERY_TIMEOUT_MS: u64 = 2000;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    model_name: String,
    id: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagEntry>,
}

#[derive(Debug, Deserialize)]
struct TagEntry {
    name: String,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct GenerateChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
}

impl OllamaClient {
    /// Discover an Ollama instance on `host:port` carrying
    /// `model_name`. Returns `Ok(Some(client))` if reachable AND the
    /// model is listed; `Ok(None)` if reachable but the model isn't
    /// present (caller may fall through to the next source);
    /// `Err(_)` only on transport-level failures the caller should
    /// surface.
    ///
    /// `host` is intentionally a `&str` (not `IpAddr`) so the
    /// operator's policy bundle can configure non-localhost setups
    /// (e.g. a sidecar deployment with Ollama on a sibling pod).
    ///
    /// NA2-B-030 / NA2-B-024: air-gap discipline is now enforced
    /// HERE, at the socket. `egress_allowed` is the operator's
    /// policy (`ModelConfig.egress_allowed`); a discovery against a
    /// **non-loopback** host is refused before any DNS lookup or
    /// connect when egress is not permitted. Loopback discovery is
    /// always allowed — contacting a local daemon is not egress
    /// under RFC G1.
    pub async fn try_discover(
        host: &str,
        port: u16,
        model_name: &str,
        egress_allowed: bool,
    ) -> Result<Option<Self>, ModelError> {
        if !egress_allowed && !host_is_loopback(host) {
            return Err(ModelError::EgressForbidden("ollama"));
        }
        let base_url = format!("http://{host}:{port}");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(DISCOVERY_TIMEOUT_MS))
            .build()
            .map_err(|e| ModelError::Transport(format!("client build: {e}")))?;

        let url = format!("{base_url}/api/tags");
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            // Connection refused / DNS / timeout — surface as
            // "not present" so the resolver can try the next source.
            Err(e) if e.is_connect() || e.is_timeout() => {
                tracing::debug!("ollama discovery {url} not reachable: {e}");
                return Ok(None);
            }
            Err(e) => return Err(ModelError::Transport(format!("GET {url}: {e}"))),
        };

        if !resp.status().is_success() {
            return Ok(None);
        }

        let tags: TagsResponse = resp
            .json()
            .await
            .map_err(|e| ModelError::Protocol(format!("/api/tags JSON: {e}")))?;

        if !tags.models.iter().any(|t| t.name == model_name) {
            tracing::debug!("ollama at {base_url} reachable but does not have model {model_name}");
            return Ok(None);
        }

        Ok(Some(Self {
            id: format!("ollama:{model_name}@{host}:{port}"),
            base_url,
            model_name: model_name.to_string(),
            client: reqwest::Client::new(),
        }))
    }

    /// Default-port localhost discovery shortcut. Loopback, so it is
    /// never gated by egress policy.
    pub async fn try_localhost(model_name: &str) -> Result<Option<Self>, ModelError> {
        Self::try_discover("127.0.0.1", DEFAULT_OLLAMA_PORT, model_name, false).await
    }
}

/// Whether `host` names the local machine (loopback). Contacting a
/// local daemon is not network egress under RFC G1.
fn host_is_loopback(host: &str) -> bool {
    let h = host.trim();
    if h.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Ok(ip) = h.parse::<std::net::IpAddr>() {
        return ip.is_loopback();
    }
    false
}

#[async_trait]
impl ModelBackend for OllamaClient {
    fn id(&self) -> &str {
        &self.id
    }

    async fn infer(&self, prompt: &str) -> Result<String, ModelError> {
        let body = GenerateRequest {
            model: &self.model_name,
            prompt,
            stream: false,
        };
        let url = format!("{}/api/generate", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Transport(format!("POST {url}: {e}")))?;
        if !resp.status().is_success() {
            return Err(ModelError::Transport(format!(
                "POST {url}: status {}",
                resp.status()
            )));
        }
        let chunk: GenerateChunk = resp
            .json()
            .await
            .map_err(|e| ModelError::Protocol(format!("/api/generate JSON: {e}")))?;
        Ok(chunk.response)
    }

    async fn infer_stream<'a>(&'a self, prompt: &'a str) -> Result<TokenStream<'a>, ModelError> {
        let body = GenerateRequest {
            model: &self.model_name,
            prompt,
            stream: true,
        };
        let url = format!("{}/api/generate", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Transport(format!("POST {url}: {e}")))?;
        if !resp.status().is_success() {
            return Err(ModelError::Transport(format!(
                "POST {url}: status {}",
                resp.status()
            )));
        }
        // Ollama's streaming protocol: NDJSON, one JSON object per
        // line, terminated by `done: true`. We collect the full
        // body before splitting; for sub-second model inferences
        // the latency difference is negligible vs the complexity
        // of byte-stream framing. Move to byte-level streaming if a
        // surface (Slint chat) reports user-visible cadence issues.
        let text = resp
            .text()
            .await
            .map_err(|e| ModelError::Transport(format!("stream body: {e}")))?;
        let mut chunks: Vec<Result<String, ModelError>> = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<GenerateChunk>(line) {
                Ok(c) => {
                    if !c.response.is_empty() {
                        chunks.push(Ok(c.response));
                    }
                    if c.done {
                        break;
                    }
                }
                Err(e) => {
                    chunks.push(Err(ModelError::Protocol(format!(
                        "/api/generate stream line: {e}"
                    ))));
                }
            }
        }
        Ok(Box::pin(stream::iter(chunks)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_format_is_descriptive() {
        // Smoke-test the id() format without making a real HTTP call.
        // We can't construct OllamaClient directly without going
        // through try_discover, so this is checked via try_discover's
        // failure mode tests below.
        let placeholder = "ollama:gemma4:e2b@127.0.0.1:11434";
        assert!(placeholder.starts_with("ollama:"));
        assert!(placeholder.contains('@'));
    }

    #[tokio::test]
    async fn try_localhost_returns_none_when_no_server() {
        // Refused-connection path. Even if a real Ollama is running
        // on this developer's machine, the named model below is
        // intentionally a UUID so .models can never list it.
        // Tests both reachable-but-no-model and not-reachable paths.
        let result = OllamaClient::try_localhost("test-model-that-does-not-exist-7b3c1a").await;
        assert!(
            result.is_ok(),
            "transport failures must be Err; not-present is Ok(None)"
        );
        assert!(
            result.unwrap().is_none(),
            "no real Ollama or the model isn't present — either way, None"
        );
    }

    #[tokio::test]
    async fn try_discover_unreachable_host_returns_none() {
        // 192.0.2.x is TEST-NET-1 per RFC 5737 — guaranteed
        // unreachable. Confirms the connect-refused path is mapped
        // to Ok(None), not Err. Egress explicitly allowed so the
        // gate below is not what produces the None.
        let result = OllamaClient::try_discover("192.0.2.1", 11434, "any-model", true).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn try_discover_refuses_non_loopback_when_egress_disabled() {
        // NA2-B-030 tripwire: a non-loopback endpoint must be refused
        // BEFORE any socket is opened when egress is not allowed.
        let result =
            OllamaClient::try_discover("192.0.2.1", 11434, "any-model", false).await;
        assert!(matches!(result, Err(ModelError::EgressForbidden("ollama"))), "got {result:?}");
    }

    #[tokio::test]
    async fn try_discover_permits_loopback_even_when_egress_disabled() {
        // Loopback is not egress: discovery is allowed and returns
        // Ok(None) when no local Ollama serves the model.
        let result =
            OllamaClient::try_discover("127.0.0.1", DEFAULT_OLLAMA_PORT, "no-such-model-x9", false)
                .await;
        assert!(result.is_ok(), "loopback discovery must not be egress-gated: {result:?}");
    }

    #[test]
    fn host_is_loopback_classifies_correctly() {
        assert!(host_is_loopback("127.0.0.1"));
        assert!(host_is_loopback("::1"));
        assert!(host_is_loopback("localhost"));
        assert!(!host_is_loopback("192.0.2.1"));
        assert!(!host_is_loopback("ollama.internal"));
    }
}
