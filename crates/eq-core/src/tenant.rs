//! Tenant resolution primitives and request extractors (ARC-04, SEC-05).

use crate::actor::ActorCtx;
use crate::config::{Config, Mode};
use crate::db::begin_read_only;
use crate::error::AppError;
use crate::ids::TenantId;
use crate::model::TenantStatus;
use crate::money::Currency;
use crate::policy::types::Locale;
use axum::extract::{ConnectInfo, FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::uri::Authority;
use ipnet::IpNet;
use moka::future::Cache;
use sqlx::PgPool;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

mod repo;

const TENANT_CACHE_TTL: Duration = Duration::from_secs(60); // technical: T-104 tenant metadata cache bound

/// ARC-04: the validated metadata needed by a request after resolving its tenant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantInfo {
    pub id: TenantId,
    pub slug: String,
    pub status: TenantStatus,
    pub timezone: String,
    pub locale: Locale,
    pub currency: Currency,
}

/// ARC-04: request tenancy is either resolved or awaiting single-tenant setup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TenantCtx {
    Resolved(Arc<TenantInfo>),
    Unprovisioned,
}

/// SEC-05: the app role reads tenant metadata only through the granted resolver functions.
#[derive(Clone)]
pub struct TenantResolver {
    pool: PgPool,
    config: Arc<Config>,
    by_slug: Cache<String, Arc<TenantInfo>>,
}

impl fmt::Debug for TenantResolver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TenantResolver")
            .field("pool", &"[configured]")
            .field("mode", &self.config.mode)
            .field("base_domain", &self.config.base_domain)
            .field("cache", &"[tenant metadata]")
            .finish()
    }
}

impl TenantResolver {
    /// SEC-05: creates a request resolver with a bounded per-slug metadata cache.
    pub fn new(pool: PgPool, config: Arc<Config>) -> Self {
        Self {
            pool,
            config,
            by_slug: Cache::builder().time_to_live(TENANT_CACHE_TTL).build(),
        }
    }

    /// Returns the validated proxy ranges used by both host and client-IP resolution.
    pub fn trusted_proxies(&self) -> &[IpNet] {
        &self.config.trusted_proxies
    }

    /// MOD-09: reads the tenant display name under the tenant's read-only RLS context.
    pub async fn public_name(&self, tenant: TenantId) -> Result<String, AppError> {
        let actor = ActorCtx::anonymous(None, None);
        let mut tx = begin_read_only(&self.pool, tenant, &actor).await?;
        let name = repo::public_name(&mut tx, tenant).await?;
        tx.commit().await?;
        Ok(name)
    }

    /// ARC-04, SEC-05: resolves the request host without reading the tenant table directly.
    pub async fn resolve(&self, parts: &Parts) -> Result<TenantCtx, AppError> {
        match self.config.mode {
            Mode::Single => self.resolve_single().await,
            Mode::Multi => self.resolve_multi(parts).await.map(TenantCtx::Resolved),
        }
    }

    async fn resolve_single(&self) -> Result<TenantCtx, AppError> {
        let tenant =
            sqlx::query!("SELECT id AS \"id!\", slug AS \"slug!\" FROM public.resolve_single_tenant()")
                .fetch_optional(&self.pool)
                .await?;
        let Some(tenant) = tenant else {
            return Ok(TenantCtx::Unprovisioned);
        };

        let info = self.resolve_by_slug(&tenant.slug).await?;
        if info.id != TenantId(tenant.id) {
            return Err(AppError::NotFound { entity: "tenant" });
        }
        Ok(TenantCtx::Resolved(info))
    }

    async fn resolve_multi(&self, parts: &Parts) -> Result<Arc<TenantInfo>, AppError> {
        let base_domain = self
            .config
            .base_domain
            .as_deref()
            .ok_or_else(|| invalid_tenant_metadata("multi mode has no base domain"))?;
        let peer = socket_peer(parts);
        let forwarded_host = peer
            .filter(|ip| is_trusted(*ip, self.trusted_proxies()))
            .and_then(|_| parts.headers.get("x-forwarded-host"))
            .and_then(|value| value.to_str().ok());
        let host = forwarded_host
            .or_else(|| {
                parts
                    .headers
                    .get(axum::http::header::HOST)
                    .and_then(|value| value.to_str().ok())
            })
            .ok_or(AppError::NotFound { entity: "tenant" })?;
        let slug = slug_from_host(host, base_domain).ok_or(AppError::NotFound { entity: "tenant" })?;
        self.resolve_by_slug(&slug).await
    }

    async fn resolve_by_slug(&self, slug: &str) -> Result<Arc<TenantInfo>, AppError> {
        if let Some(info) = self.by_slug.get(slug).await {
            return Ok(info);
        }

        let row = sqlx::query_as!(
            TenantInfoRow,
            "SELECT id AS \"id!\", slug AS \"slug!\", status AS \"status!\", \
                    timezone AS \"timezone!\", locale AS \"locale!\", currency AS \"currency!\" \
             FROM public.resolve_tenant_info_by_slug($1)",
            slug
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound { entity: "tenant" })?;
        let info = Arc::new(TenantInfo::try_from(row).map_err(invalid_tenant_metadata)?);
        self.by_slug.insert(String::from(slug), Arc::clone(&info)).await;
        Ok(info)
    }
}

/// ARC-04: extracts the tenant context from the request host and server state.
impl<S> FromRequestParts<S> for TenantCtx
where
    S: Send + Sync,
    TenantResolver: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(context) = parts.extensions.get::<Self>() {
            return Ok(context.clone());
        }
        TenantResolver::from_ref(state).resolve(parts).await
    }
}

/// ARC-04: requires a provisioned tenant for handlers that read or mutate tenant data.
#[derive(Debug)]
pub struct RequireTenant(pub Arc<TenantInfo>);

impl<S> FromRequestParts<S> for RequireTenant
where
    S: Send + Sync,
    TenantResolver: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match TenantCtx::from_request_parts(parts, state).await? {
            TenantCtx::Resolved(info) => Ok(Self(info)),
            TenantCtx::Unprovisioned => Err(AppError::SetupRequired),
        }
    }
}

/// ARC-04: the anonymous request actor used before authentication upgrades its role.
#[derive(Debug)]
pub struct RequestActor(pub ActorCtx);

impl<S> FromRequestParts<S> for RequestActor
where
    S: Send + Sync,
    TenantResolver: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let resolver = TenantResolver::from_ref(state);
        let ip = client_ip(parts, resolver.trusted_proxies());
        let request_id = parts.extensions.get::<Uuid>().copied();
        Ok(Self(ActorCtx::anonymous(ip, request_id)))
    }
}

/// SBD-20: trusts forwarded client addresses only when the socket peer is configured as a proxy.
pub fn client_ip(parts: &Parts, trusted_proxies: &[IpNet]) -> Option<IpAddr> {
    let peer = socket_peer(parts)?;
    if !is_trusted(peer, trusted_proxies) {
        return Some(peer);
    }

    let Some(forwarded) = parts
        .headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
    else {
        return Some(peer);
    };
    let chain = forwarded
        .split(',')
        .map(str::trim)
        .map(IpAddr::from_str)
        .collect::<Result<Vec<_>, _>>();
    let Ok(mut chain) = chain else {
        return Some(peer);
    };
    chain.push(peer);

    chain
        .iter()
        .rev()
        .copied()
        .find(|address| !is_trusted(*address, trusted_proxies))
        .or_else(|| chain.first().copied())
}

fn socket_peer(parts: &Parts) -> Option<IpAddr> {
    parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(peer)| peer.ip())
}

fn is_trusted(peer: IpAddr, trusted_proxies: &[IpNet]) -> bool {
    trusted_proxies.iter().any(|network| network.contains(&peer))
}

fn slug_from_host(host: &str, base_domain: &str) -> Option<String> {
    let authority = Authority::from_str(host).ok()?;
    let host = authority.host().trim_end_matches('.').to_ascii_lowercase();
    let base_domain = base_domain.trim_end_matches('.').to_ascii_lowercase();
    let suffix = format!(".{base_domain}");
    let slug = host.strip_suffix(&suffix)?;
    if slug.is_empty() || slug.contains('.') {
        return None;
    }
    Some(String::from(slug))
}

struct TenantInfoRow {
    id: Uuid,
    slug: String,
    status: String,
    timezone: String,
    locale: String,
    currency: String,
}

impl TryFrom<TenantInfoRow> for TenantInfo {
    type Error = &'static str;

    fn try_from(row: TenantInfoRow) -> Result<Self, Self::Error> {
        let status = match row.status.as_str() {
            "active" => TenantStatus::Active,
            "suspended" => TenantStatus::Suspended,
            "closed" => TenantStatus::Closed,
            _ => return Err("invalid tenant status"),
        };
        let locale = match row.locale.as_str() {
            "en" => Locale::En,
            "id" => Locale::Id,
            _ => return Err("invalid tenant locale"),
        };
        let currency = Currency::from_code(row.currency.trim()).ok_or("invalid tenant currency")?;
        Ok(Self {
            id: TenantId(row.id),
            slug: row.slug,
            status,
            timezone: row.timezone,
            locale,
            currency,
        })
    }
}

fn invalid_tenant_metadata(error: impl fmt::Display) -> AppError {
    AppError::Internal(std::io::Error::other(error.to_string()).into())
}

#[cfg(test)]
mod tests {
    use super::{client_ip, slug_from_host};
    use axum::extract::ConnectInfo;
    use axum::http::{HeaderValue, Request};
    use ipnet::IpNet;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn multi_host_strips_port_and_normalizes_case() {
        assert_eq!(
            slug_from_host("AYU.App.Test:8080", "app.test").as_deref(),
            Some("ayu")
        );
    }

    #[test]
    fn multi_host_rejects_hosts_outside_the_base_domain() {
        assert_eq!(slug_from_host("ayu.other.test", "app.test"), None);
        assert_eq!(slug_from_host("nested.ayu.app.test", "app.test"), None);
    }

    #[test]
    fn client_ip_ignores_forwarded_values_without_a_trusted_peer() {
        let mut parts = Request::new(()).into_parts().0;
        parts
            .headers
            .insert("x-forwarded-for", HeaderValue::from_static("198.51.100.1"));
        let peer = SocketAddr::from(([192, 0, 2, 25], 443));
        parts.extensions.insert(ConnectInfo(peer));

        assert_eq!(client_ip(&parts, &[]), Some(peer.ip()));
    }

    #[test]
    fn client_ip_walks_back_through_the_trusted_proxy_chain() -> Result<(), &'static str> {
        let network = IpNet::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8)
            .map_err(|_| "the fixed test network is valid")?;
        let mut parts = Request::new(()).into_parts().0;
        parts.headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("198.51.100.4, 10.0.0.1"),
        );
        parts
            .extensions
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 2], 443))));

        let client = client_ip(&parts, &[network]);
        assert_eq!(client, Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 4))));
        Ok(())
    }
}
