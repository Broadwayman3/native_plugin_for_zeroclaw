use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use governor::{Quota, RateLimiter};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

/// Shared rate limiter state keyed by IP address.
pub type SharedLimiter = Arc<governor::DefaultKeyedRateLimiter<SocketAddr>>;

/// Creates a shared rate limiter with the given requests-per-second limit.
pub fn create_rate_limiter(rps: u32) -> SharedLimiter {
    let quota = Quota::per_second(NonZeroU32::new(rps).unwrap_or(NonZeroU32::new(10).unwrap()));
    Arc::new(RateLimiter::keyed(quota))
}

/// Extracts the client IP from X-Forwarded-For, X-Real-IP, or the socket address.
pub(crate) fn extract_client_ip(headers: &HeaderMap, addr: Option<SocketAddr>) -> SocketAddr {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(first) = s.split(',').next() {
                if let Ok(ip) = first.trim().parse::<SocketAddr>() {
                    return ip;
                }
            }
        }
    }
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(s) = real_ip.to_str() {
            if let Ok(ip) = s.trim().parse::<SocketAddr>() {
                return ip;
            }
        }
    }
    addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)))
}

/// Tower layer for rate limiting per client IP.
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: SharedLimiter,
}

impl RateLimitLayer {
    pub fn new(limiter: SharedLimiter) -> Self {
        Self { limiter }
    }
}

impl<S> tower::Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: SharedLimiter,
}

impl<S, ReqBody> tower::Service<axum::http::Request<ReqBody>> for RateLimitService<S>
where
    S: tower::Service<axum::http::Request<ReqBody>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let headers = req.headers().clone();
        let addr = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0);
        let client_ip = extract_client_ip(&headers, addr);

        if self.limiter.check_key(&client_ip).is_err() {
            tracing::warn!(client_ip = %client_ip, "Rate limit exceeded");
            return Box::pin(async {
                Ok((
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": "Rate limit exceeded. Try again later."
                    })),
                )
                    .into_response())
            });
        }

        let mut inner = self.inner.clone();
        Box::pin(async move { inner.call(req).await })
    }
}

/// Auth configuration for protected routes.
#[derive(Clone, Default)]
pub struct AuthConfig {
    /// Telegram Bot API secret token (from X-Telegram-Bot-Api-Secret-Token header).
    pub telegram_bot_secret_token: Option<String>,
    /// Valid API keys for external clients (from X-API-Key header).
    pub api_keys: Vec<String>,
    /// Manager Telegram user ID for manager-only actions (from X-Telegram-User-Id header).
    pub manager_telegram_id: Option<i64>,
}

/// Tower layer for auth on mutating routes.
#[derive(Clone)]
pub struct AuthLayer {
    config: AuthConfig,
}

impl AuthLayer {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }
}

impl<S> tower::Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    config: AuthConfig,
}

impl<S, ReqBody> tower::Service<axum::http::Request<ReqBody>> for AuthService<S>
where
    S: tower::Service<axum::http::Request<ReqBody>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let headers = req.headers().clone();

        // If no auth config, allow all (open mode for development)
        if self.config.telegram_bot_secret_token.is_none() && self.config.api_keys.is_empty() {
            let mut inner = self.inner.clone();
            return Box::pin(async move { inner.call(req).await });
        }

        // Check Telegram Bot Secret Token
        if let Some(expected) = &self.config.telegram_bot_secret_token {
            if let Some(actual) = headers.get("x-telegram-bot-api-secret-token") {
                if actual.to_str().ok() == Some(expected.as_str()) {
                    let mut inner = self.inner.clone();
                    return Box::pin(async move { inner.call(req).await });
                }
            }
        }

        // Check API Key
        if !self.config.api_keys.is_empty() {
            if let Some(api_key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
                if self.config.api_keys.contains(&api_key.to_string()) {
                    let mut inner = self.inner.clone();
                    return Box::pin(async move { inner.call(req).await });
                }
            }
        }

        tracing::warn!("Unauthorized request — missing or invalid auth headers");
        Box::pin(async {
            Ok((
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": "Unauthorized. Provide valid X-Telegram-Bot-Api-Secret-Token or X-API-Key header."
                })),
            )
                .into_response())
        })
    }
}

/// Tower layer for manager-only routes (refund approve/reject).
#[derive(Clone)]
pub struct ManagerLayer {
    manager_id: Option<i64>,
}

impl ManagerLayer {
    pub fn new(manager_id: Option<i64>) -> Self {
        Self { manager_id }
    }
}

impl<S> tower::Layer<S> for ManagerLayer {
    type Service = ManagerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ManagerService {
            inner,
            manager_id: self.manager_id,
        }
    }
}

#[derive(Clone)]
pub struct ManagerService<S> {
    inner: S,
    manager_id: Option<i64>,
}

impl<S, ReqBody> tower::Service<axum::http::Request<ReqBody>> for ManagerService<S>
where
    S: tower::Service<axum::http::Request<ReqBody>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // If no manager configured, allow all (development mode)
        let Some(expected_id) = self.manager_id else {
            let mut inner = self.inner.clone();
            return Box::pin(async move { inner.call(req).await });
        };

        let headers = req.headers().clone();
        let user_id = headers
            .get("x-telegram-user-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());

        match user_id {
            Some(id) if id == expected_id => {
                let mut inner = self.inner.clone();
                Box::pin(async move { inner.call(req).await })
            }
            _ => {
                tracing::warn!(
                    expected_manager = expected_id,
                    actual_user = ?user_id,
                    "Manager-only action rejected"
                );
                Box::pin(async {
                    Ok((
                        StatusCode::FORBIDDEN,
                        axum::Json(serde_json::json!({
                            "error": "Forbidden. This action requires manager authorization."
                        })),
                    )
                        .into_response())
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};

    fn make_header(name: HeaderName, val: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(name, HeaderValue::from_str(val).unwrap());
        h
    }

    #[test]
    fn test_extract_ip_x_forwarded_for_single() {
        let headers = make_header(HeaderName::from_static("x-forwarded-for"), "1.2.3.4:8080");
        let addr = extract_client_ip(&headers, None);
        assert_eq!(addr.ip(), "1.2.3.4".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_ip_x_forwarded_for_multiple() {
        let headers = make_header(
            HeaderName::from_static("x-forwarded-for"),
            "1.2.3.4:8080, 5.6.7.8:9090",
        );
        let addr = extract_client_ip(&headers, None);
        assert_eq!(addr.ip(), "1.2.3.4".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_ip_x_real_ip() {
        let headers = make_header(HeaderName::from_static("x-real-ip"), "10.0.0.1:0");
        let addr = extract_client_ip(&headers, None);
        assert_eq!(addr.ip(), "10.0.0.1".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_ip_fallback_to_socket_addr() {
        let headers = HeaderMap::new();
        let fallback = SocketAddr::from(([192, 168, 1, 1], 3000));
        let addr = extract_client_ip(&headers, Some(fallback));
        assert_eq!(addr, fallback);
    }

    #[test]
    fn test_extract_ip_fallback_to_loopback() {
        let headers = HeaderMap::new();
        let addr = extract_client_ip(&headers, None);
        assert_eq!(addr.ip(), "127.0.0.1".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_ip_malformed_x_forwarded_for() {
        let headers = make_header(HeaderName::from_static("x-forwarded-for"), "not-an-ip");
        let fallback = SocketAddr::from(([10, 0, 0, 1], 0));
        let addr = extract_client_ip(&headers, Some(fallback));
        assert_eq!(addr, fallback);
    }

    #[test]
    fn test_extract_ip_x_forwarded_for_takes_precedence() {
        let mut headers = make_header(HeaderName::from_static("x-forwarded-for"), "1.2.3.4:8080");
        headers.insert(
            HeaderName::from_static("x-real-ip"),
            HeaderValue::from_str("10.0.0.1:0").unwrap(),
        );
        let addr = extract_client_ip(&headers, None);
        assert_eq!(addr.ip(), "1.2.3.4".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn test_extract_ip_ipv6_address() {
        let headers = make_header(HeaderName::from_static("x-forwarded-for"), "[::1]:8080");
        let addr = extract_client_ip(&headers, None);
        assert_eq!(addr.ip(), "::1".parse::<std::net::IpAddr>().unwrap());
    }
}
