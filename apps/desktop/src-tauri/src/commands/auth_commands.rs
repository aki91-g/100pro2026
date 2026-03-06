use std::sync::Arc;
use serde::Deserialize;
use tauri::State;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use crate::models::user::{LocalUser, LocalUserUpdate};
use crate::models::session::LocalSession;
use crate::repositories::user_repo::UserRepository;
use crate::repositories::session_repo::SessionRepository;
use crate::repositories::profile_repo::ProfileRepository;
use crate::state::AppState;

const DEFAULT_USERNAME: &str = "Unknown User";

#[derive(Debug, Deserialize)]
struct SupabaseLoginResponse {
    user: SupabaseUser,
}

#[derive(Debug, Deserialize)]
struct SupabaseUser {
    id: Uuid,
}

async fn authenticate_with_supabase(email: &str, password: &str) -> Result<Uuid, String> {
    let supabase_url = std::env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL is not set".to_string())?;
    let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY")
        .map_err(|_| "SUPABASE_ANON_KEY is not set".to_string())?;

    let endpoint = format!(
        "{}/auth/v1/token?grant_type=password",
        supabase_url.trim_end_matches('/')
    );

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    let response = client
        .post(endpoint)
        .header("apikey", &supabase_anon_key)
        .header("Authorization", format!("Bearer {}", supabase_anon_key))
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .map_err(|e| format!("Supabase auth request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Supabase auth failed ({}): {}", status, body));
    }

    let payload: SupabaseLoginResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Supabase auth response: {}", e))?;

    Ok(payload.user.id)
}

async fn fetch_username_from_profiles(
    user_id: &uuid::Uuid,
    profile_repo: &Arc<tokio::sync::RwLock<Option<Arc<dyn ProfileRepository>>>>,
) -> String {
    let from_managed_repo = {
        let guard = profile_repo.read().await;
        if let Some(repo) = guard.as_ref() {
            match repo.get_profile(*user_id).await {
                Ok(Some(profile)) => profile.username,
                Ok(None) => None,
                Err(e) => {
                    eprintln!("⚠️ Failed to fetch profile from managed repo: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    if let Some(name) = from_managed_repo {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        match PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect(&database_url)
            .await
        {
            Ok(pool) => {
                let query_result = sqlx::query_scalar::<_, Option<String>>(
                    "SELECT username FROM public.profiles WHERE id = $1::uuid"
                )
                .bind(user_id)
                .persistent(false)
                .fetch_optional(&pool)
                .await;

                if let Ok(Some(Some(name))) = query_result {
                    let trimmed = name.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ Could not connect to Postgres for profile lookup: {}", e);
            }
        }
    }

    DEFAULT_USERNAME.to_string()
}

/// Response DTO for login - sanitized to exclude sensitive fields
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    pub id: String,
    pub username: String,
}

/// Session DTO exposed to frontend (sanitized).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SafeSession {
    pub user_id: String,
    pub username: String,
    pub last_login: Option<String>,
}

impl From<LocalSession> for SafeSession {
    fn from(session: LocalSession) -> Self {
        SafeSession {
            user_id: session.user_id.to_string(),
            username: session.username,
            last_login: session.last_login,
        }
    }
}

impl From<LocalUser> for LoginResponse {
    fn from(user: LocalUser) -> Self {
        LoginResponse {
            id: user.id.to_string(),
            username: user.username,
        }
    }
}

/// Auth command: Login with Supabase and save to local DB
#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    user_repo: State<'_, Arc<dyn UserRepository>>,
    session_repo: State<'_, Arc<dyn SessionRepository>>,
    profile_repo: State<'_, Arc<tokio::sync::RwLock<Option<Arc<dyn ProfileRepository>>>>>,
    app_state: State<'_, AppState>,
) -> Result<LoginResponse, String> {
    if email.trim().is_empty() || password.is_empty() {
        return Err("Email and password are required".to_string());
    }

    // 1. Authenticate with Supabase Auth (email/password)
    let user_id = authenticate_with_supabase(email.trim(), &password).await?;

    // 2. Resolve username from public.profiles
    let username = fetch_username_from_profiles(&user_id, profile_repo.inner()).await;

    // 3. Save user to local SQLite (local_user table)
    let user_update = LocalUserUpdate {
        id: user_id,
        username: username.clone(),
    };
    
    user_repo.upsert_user(&user_update)
        .await
        .map_err(|e| format!("Failed to save user locally: {}", e))?;

    // 4. Save session to local_session table
    session_repo.save_session(&user_id, &username, None)
        .await
        .map_err(|e| format!("Failed to save session: {}", e))?;

    // 5. Set AppState to logged-in user
    app_state.set_user(user_id, username.clone()).await;
    
    
    // 6. Return sanitized user data. If lookup fails after successful login/session writes,
    // do not fail the command; return a safe fallback from known values.
    match user_repo.get_user_by_id(&user_id).await {
        Ok(Some(user)) => Ok(LoginResponse::from(user)),
        Ok(None) => {
            eprintln!("⚠️ Login succeeded but local user lookup returned None; returning fallback response");
            Ok(LoginResponse {
                id: user_id.to_string(),
                username,
            })
        }
        Err(e) => {
            eprintln!("⚠️ Login succeeded but failed to retrieve local user: {}. Returning fallback response", e);
            Ok(LoginResponse {
                id: user_id.to_string(),
                username,
            })
        }
    }
}

/// Logout command: Clear session and AppState
#[tauri::command]
pub async fn logout(
    session_repo: State<'_, Arc<dyn SessionRepository>>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. Always clear in-memory auth state first.
    app_state.clear_user_id().await;

    // 2. Then clear persisted session.
    session_repo.clear_session()
        .await
        .map_err(|e| format!("Failed to clear session: {}", e))?;
    
    Ok(())
}

/// Get active session: Check local_session table for active session
#[tauri::command]
pub async fn get_active_session(
    session_repo: State<'_, Arc<dyn SessionRepository>>,
) -> Result<Option<SafeSession>, String> {
    session_repo.get_active_session()
        .await
        .map(|maybe_session| maybe_session.map(SafeSession::from))
        .map_err(|e| format!("Failed to get active session: {}", e))
}

/// Auto-login: Check local_session table and restore session
#[tauri::command]
pub async fn auto_login(
    session_repo: State<'_, Arc<dyn SessionRepository>>,
    app_state: State<'_, AppState>,
) -> Result<Option<SafeSession>, String> {
    match session_repo.get_active_session().await {
        Ok(Some(session)) => {
            // Set the AppState to this user
            app_state.set_user(session.user_id, session.username.clone()).await;
            
            // Update last_login timestamp
            session_repo.update_last_login()
                .await
                .map_err(|e| format!("Failed to update last login: {}", e))?;
            
            Ok(Some(SafeSession::from(session)))
        }
        Ok(None) => {
            app_state.clear_user_id().await;
            Ok(None)
        },
        Err(e) => Err(format!("Failed to check for active session: {}", e))
    }
}
