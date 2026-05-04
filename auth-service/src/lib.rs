use base64::{Engine, engine::general_purpose};
use dotenv::dotenv;
use jsonwebtoken::{Algorithm, Validation, jwk::JwkSet};
use reqwest::{Client, StatusCode};
use std::error::Error as StdError;
use utility_helpers::types::{EnvVarConfig, GoogleClaims ,  UnifiedClaims ,AuthProvider  };
use uuid::Uuid;
use serde::Deserialize;

use token_services::Claims;
use types::{
    AuthenticateUserError, GoogleClaimsError, GoogleTokenInfoResponse, SessionTokenClaims,
};
use utility_helpers::{ log_error, log_info,};
use db_service::schema::users::User;


pub mod token_services;
pub mod types;

#[derive(Clone)]
pub struct AuthService {
    pub client: Client,
    pub pool: sqlx::PgPool,
    pub env_var_config: EnvVarConfig,
}

#[derive(Debug, Deserialize)]
struct FirebaseClaims {
    sub: String,
    email: Option<String>,
    exp: usize,
    iss: String,
    aud: String,
}
impl AuthService {
    pub fn new(pg_pool: sqlx::PgPool) -> Result<Self, Box<dyn StdError>> {
        dotenv().ok();

        let client = Client::new();
        let env_var_config = EnvVarConfig::new()?;

        let auth_service = AuthService {
            client,
            env_var_config,
            pool: pg_pool,
        };

        Ok(auth_service)
    }

    pub fn get_claims(data: String, exp: usize) -> Claims {
        Claims::new(data, exp)
    }

    pub async fn get_google_claims(
        &self,
        id_token: &String,
    ) -> Result<GoogleClaims, GoogleClaimsError> {
        let id_token_component = id_token.split(".").collect::<Vec<_>>();

        if id_token_component.len() != 3 {
            return Err(GoogleClaimsError::InvalidTokenId);
        }

        let client = self.client.clone();

        let header_json = general_purpose::URL_SAFE_NO_PAD
            .decode(id_token_component[0])
            .map_err(|_| GoogleClaimsError::FailedToDecodeHeader)?;
        let header: serde_json::Value = serde_json::from_slice(&header_json)
            .map_err(|_| GoogleClaimsError::FailedToGetHeaderSlice)?;
        let kid = header
            .get("kid")
            .and_then(|k| k.as_str())
            .ok_or(GoogleClaimsError::MissingKid)?;

        let jwks_url = "https://www.googleapis.com/oauth2/v3/certs";
        let auth_url = format!(
            "https://oauth2.googleapis.com/tokeninfo?id_token={}",
            id_token
        );

        let (token_auth_response, jwk_sets_response) = tokio::join!(
            get_token_auth_resp(&client, &auth_url, &self.env_var_config.google_client_id),
            get_jwk_set_resp_string(&client, jwks_url)
        );

        let (token_auth_response, jwk_sets_response) =
            match (token_auth_response, jwk_sets_response) {
                (Ok(token_auth_response), Ok(jwk_sets_response)) => {
                    (token_auth_response, jwk_sets_response)
                }
                (Err(e), _) => return Err(e),
                (_, Err(e)) => return Err(e),
            };

        let jwk_set: JwkSet = serde_json::from_str(&jwk_sets_response)
            .map_err(|_| GoogleClaimsError::FailedToSetJwkSetFromGoogle)?;

        let google_pk_kid = jwk_set
            .keys
            .iter()
            .find(|key| key.common.key_id.as_deref() == Some(kid))
            .ok_or(GoogleClaimsError::KeyNotFound)?;

        if token_auth_response.kid.unwrap() != kid
            || google_pk_kid.common.key_id != Some(kid.to_string())
        {
            return Err(GoogleClaimsError::InvalidTokenId);
        }

        Ok(GoogleClaims {
            sub: token_auth_response.sub,
            email: token_auth_response.email,
            name: token_auth_response.name,
            picture: token_auth_response.picture,
            exp: token_auth_response
                .exp
                .parse::<usize>()
                .map_err(|_| GoogleClaimsError::FailedToDecodeRsaComponents)?,
        })
    }

pub async fn verify_identity_token(
    &self,
    token: &str,
) -> Result<UnifiedClaims, AuthenticateUserError> {
    // Decode JWT payload to read issuer
    let payload = token.split('.').nth(1)
        .ok_or(AuthenticateUserError::InvalidToken)?;
    let decoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AuthenticateUserError::InvalidToken)?;
    let payload_json: serde_json::Value = serde_json::from_slice(&decoded_payload)
        .map_err(|_| AuthenticateUserError::InvalidToken)?;
    let issuer = payload_json.get("iss")
        .and_then(|v| v.as_str())
        .ok_or(AuthenticateUserError::InvalidToken)?;

    // Firebase Token
    if issuer.contains("securetoken.google.com") {
        let project_id = std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default();

        let header = jsonwebtoken::decode_header(token)
            .map_err(|_| AuthenticateUserError::InvalidToken)?;
        let kid = header.kid.ok_or(AuthenticateUserError::InvalidToken)?;

        let client = reqwest::Client::new();
        let certs: std::collections::HashMap<String, String> = client
            .get("https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com")
            .send()
            .await
            .map_err(|_| AuthenticateUserError::InvalidToken)?
            .json()
            .await
            .map_err(|_| AuthenticateUserError::InvalidToken)?;

        let cert = certs.get(&kid)
            .ok_or(AuthenticateUserError::InvalidToken)?;

        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        
        // ✅ Set audience (expects slice)
        validation.set_audience(&[project_id.as_str()]);
        
        // ✅ FIX: iss should be a HashSet<String>, not Option<String>
        let mut issuers = std::collections::HashSet::new();
        issuers.insert(format!("https://securetoken.google.com/{}", project_id));
        validation.iss = Some(issuers);  // iss is Option<HashSet<String>>

        let decoded = jsonwebtoken::decode::<FirebaseClaims>(
            token,
            &jsonwebtoken::DecodingKey::from_rsa_pem(cert.as_bytes())
                .map_err(|_| AuthenticateUserError::InvalidToken)?,
            &validation,
        )
        .map_err(|_| AuthenticateUserError::InvalidToken)?;

   // ✅ Extract username from email
    let email = decoded.claims.email.unwrap_or_default();
    let name = if email.is_empty() {
        "User".to_string()
    } else {
        // Take part before @ sign
        email.split('@').next().unwrap_or("User").to_string()
    };

        Ok(UnifiedClaims {
            sub: decoded.claims.sub,
             email, 
            name: name,
            picture: None,
            exp: decoded.claims.exp,
provider: AuthProvider::Firebase,

        })

    // Google OAuth Token
    } else if issuer.contains("accounts.google.com") {
        let google_claims = self
            .get_google_claims(&token.to_string())
            .await
            .map_err(|_| AuthenticateUserError::InvalidToken)?;

        Ok(UnifiedClaims {
            sub: google_claims.sub,
            email: google_claims.email,
            name: google_claims.name,
            picture: Some(google_claims.picture),
            exp: google_claims.exp,
            provider: AuthProvider::Google,
        })
    } else {
        Err(AuthenticateUserError::InvalidToken)
    }
}



fn generate_session_token(
    &self,
    google_claims: &UnifiedClaims,
    user_id: Uuid,
) -> Result<String, Box<dyn StdError>> {
    let current_time = chrono::Utc::now().timestamp() as usize;

    // Check if this user should be admin
    let admin_email = std::env::var("ADMIN_EMAIL").unwrap_or_default();
    let is_admin = google_claims.email == admin_email;

    let session_claims = SessionTokenClaims {
        user_id,
        google_sub: google_claims.sub.clone(),
        email: Some(google_claims.email.clone()),
        exp: current_time + 60 * 60 * 24 * 30, // 30 days
        is_admin, // ← set based on env check
    };

    let session_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &session_claims,
        &jsonwebtoken::EncodingKey::from_secret(self.env_var_config.jwt_secret.as_ref()),
    )
    .map_err(|_| "Failed to encode session token".to_string())?;

    Ok(session_token)
}


    pub fn verify_session_token(
        &self,
        session_token: &str,
    ) -> Result<SessionTokenClaims, Box<dyn StdError>> {
        let validation = Validation::new(Algorithm::HS256);

        let token_data = jsonwebtoken::decode::<SessionTokenClaims>(
            session_token,
            &jsonwebtoken::DecodingKey::from_secret(self.env_var_config.jwt_secret.as_ref()),
            &validation,
        )
        .map_err(|e| {
            log_info!("Error decoding session token: {:?}", e);
            "Failed to decode session token".to_string()
        })?;
        let claims = token_data.claims;
        let current_time = chrono::Utc::now().timestamp() as usize;
        if claims.exp < current_time {
            return Err("Session token expired".into());
        }

        Ok(claims)
    }


pub async fn authenticate_user(
    &self,
    id_token: &str,
    referral_code: Option<String>,
) -> Result<(Uuid, String, bool, bool), AuthenticateUserError> 
{
    // 1️⃣ Verify Google/Firebase token
    let claims = self
        .verify_identity_token(id_token)
        .await
        .map_err(|_| AuthenticateUserError::InvalidToken)?;

    // 2️⃣ Create or update user
    let (user, is_new_user) =
        db_service::schema::users::User::create_or_update_existing_user(
            &self.pool,
            &claims,
        )
        .await
        .map_err(|_| AuthenticateUserError::FailedToInsertUser)?;

    let mut was_referred = false;

    // 3️⃣ Apply referral ONLY for new users
    if is_new_user {
        if let Some(code) = referral_code {
            let referrer = User::find_by_referral_code(&self.pool, &code)
                .await
                .map_err(|_| AuthenticateUserError::FailedToInsertUser)?;

            if let Some(referrer) = referrer {
                if referrer.id != user.id {
                    User::set_referred_by(&self.pool, user.id, referrer.id)
                        .await
                        .map_err(|_| AuthenticateUserError::FailedToInsertUser)?;
                    was_referred = true;
                }
            } else {
                return Err(AuthenticateUserError::InvalidReferral);
            }
        }
    }

    // 4️⃣ Generate session token
    let session_token = self
        .generate_session_token(&claims, user.id)
        .map_err(|_| AuthenticateUserError::FailedToGenerateSessionToken)?;

    Ok((user.id, session_token, is_new_user, was_referred))
}

}

async fn get_token_auth_resp(
    client: &Client,
    url: &str,
    google_client_id: &str,
) -> Result<GoogleTokenInfoResponse, GoogleClaimsError> {
    let req = client
        .get(url)
        .send()
        .await
        .map_err(|_| GoogleClaimsError::FailedToValidateTokenFromGoogle)?;

    if req.status() != StatusCode::OK {
        return Err(GoogleClaimsError::ExpiredOrInvalidToken);
    }

    let response = req
        .json::<GoogleTokenInfoResponse>()
        .await
        .map_err(|_| GoogleClaimsError::FailedToDecodeAuthResponseFromGoogle)?;

    if response.iss != "accounts.google.com" && response.iss != "https://accounts.google.com" {
        return Err(GoogleClaimsError::InvalidIssuer);
    }

    if response.aud != google_client_id {
        return Err(GoogleClaimsError::InvalidClientId);
    }

    if let Some(kid) = &response.kid {
        if kid.is_empty() {
            return Err(GoogleClaimsError::MissingKid);
        }
    } else {
        return Err(GoogleClaimsError::MissingKid);
    }

    Ok(response)
}

async fn get_jwk_set_resp_string(client: &Client, url: &str) -> Result<String, GoogleClaimsError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| GoogleClaimsError::FailedToGetKeyFromGoogle)?
        .text()
        .await
        .map_err(|_| GoogleClaimsError::FailedToDecodeKeyFromGoogle)?;

    Ok(response)
}
 