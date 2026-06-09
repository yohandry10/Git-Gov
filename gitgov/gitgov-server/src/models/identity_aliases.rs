use super::*;

// IDENTITY ALIASES — T3.B
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAlias {
    pub canonical_login: String,
    pub alias_login: String,
    #[serde(default)]
    pub org_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityAliasRequest {
    pub canonical: String,
    pub alias: String,
    #[serde(default)]
    pub org_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityAliasResponse {
    pub canonical_login: String,
    pub alias_login: String,
    pub created: bool,
}

// ============================================================================

// IDENTITY ALIAS HELPERS
// ============================================================================

/// Expands a canonical login to include all associated alias logins.
///
/// Returns a `Vec` with `canonical` as the first element followed by every
/// `alias_login` where `alias.canonical_login == canonical`.
///
/// Used when querying events: a developer who has multiple GitHub accounts
/// linked via identity aliases should have their events surfaced by searching
/// for the canonical login.
pub fn expand_login_aliases(canonical: &str, aliases: &[IdentityAlias]) -> Vec<String> {
    let mut logins = vec![canonical.to_string()];
    for alias in aliases {
        if alias.canonical_login == canonical {
            logins.push(alias.alias_login.clone());
        }
    }
    logins
}
