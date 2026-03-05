use std::sync::Arc;
use serde::Deserialize;
use tauri::State;
use sqlx::postgres::PgPoolOptions;
use crate::models::user::{LocalUser, LocalUserUpdate};
use crate::repositories::user_repo::UserRepository;
use crate::repositories::profile_repo::ProfileRepository;
use crate::state::AppState;
use crate::services::item_service::ItemService;

const DEFAULT_USERNAME: &str = "Unknown User";

#[derive(Debug, Deserialize)]
struct SupabaseLoginResponse {
    user: SupabaseUser,
}

#[derive(Debug, Deserialize)]
struct SupabaseUser {
    id: String,
}

async fn authenticate_with_supabase(email: &str, password: &str) -> Result<String, String> {
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
    user_id: &str,
    profile_repo: &Arc<tokio::sync::RwLock<Option<Arc<dyn ProfileRepository>>>>,
) -> String {
    let from_managed_repo = {
        let guard = profile_repo.read().await;
        if let Some(repo) = guard.as_ref() {
            match repo.get_profile(user_id).await {
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

impl From<LocalUser> for LoginResponse {
    fn from(user: LocalUser) -> Self {
        LoginResponse {
            id: user.id,
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
    profile_repo: State<'_, Arc<tokio::sync::RwLock<Option<Arc<dyn ProfileRepository>>>>>,
    app_state: State<'_, AppState>,
    item_service: State<'_, Arc<ItemService>>,
) -> Result<LoginResponse, String> {
    if email.trim().is_empty() || password.is_empty() {
        return Err("Email and password are required".to_string());
    }

    // 1. Authenticate with Supabase Auth (email/password)
    let user_id = authenticate_with_supabase(email.trim(), &password).await?;

    // 2. Resolve username from public.profiles
    let username = fetch_username_from_profiles(&user_id, profile_repo.inner()).await;

    // 3. Save user to local SQLite
    let user_update = LocalUserUpdate {
        id: user_id.clone(),
        username: username.clone(),
    };
    
    user_repo.upsert_user(&user_update)
        .await
        .map_err(|e| format!("Failed to save user locally: {}", e))?;

    // 4. Set AppState to logged-in user
    app_state.set_user(user_id.clone(), username.clone()).await;
    
    // 5. Auto-claim orphaned items
    match item_service.claim_offline_items(&user_id).await {
        Ok(count) => {
            if count > 0 {
                println!("✅ Migrated {} orphaned items", count);
                
                // 6. Trigger background sync to push claimed items to Supabase
                let item_service_bg = item_service.inner().clone();
                let user_id_bg = user_id.clone();
                tokio::spawn(async move {
                    match item_service_bg.sync_local_to_remote(&user_id_bg).await {
                        Ok(synced_count) => {
                            println!("⬆️ Synced {} local items to remote after migration", synced_count);
                        }
                        Err(_) => {
                            eprintln!("⚠️ Failed to sync local items to remote after migration");
                        }
                    }
                });
            } else {
                println!("✓ No orphaned items found");
            }
        }
        Err(e) => {
            eprintln!("⚠️ Failed to claim orphaned items: {}", e);
        }
    }
    
    // 7. Return sanitized user data
    let user = user_repo.get_user_by_id(&user_id)
        .await
        .map_err(|e| format!("Failed to retrieve user: {}", e))?
        .ok_or_else(|| "User not found after login".to_string())?;
    
    Ok(LoginResponse::from(user))
}

/// Logout command: Deactivate local user and clear AppState
#[tauri::command]
pub async fn logout(
    user_repo: State<'_, Arc<dyn UserRepository>>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. Get current user
    let user_id = app_state.get_user_id().await?;
    
    // 2. Deactivate user in local DB
    user_repo.deactivate_user(&user_id)
        .await
        .map_err(|e| format!("Failed to deactivate user: {}", e))?;
    
    // 3. Clear AppState
    app_state.clear_user_id().await;
    
    Ok(())
}

/// Get the currently active local user
#[tauri::command]
pub async fn get_active_local_user(
    user_repo: State<'_, Arc<dyn UserRepository>>,
) -> Result<Option<LocalUser>, String> {
    user_repo.get_active_user()
        .await
        .map_err(|e| format!("Failed to get active user: {}", e))
}

/// Auto-login: Check local_user table and restore session
#[tauri::command]
pub async fn auto_login(
    user_repo: State<'_, Arc<dyn UserRepository>>,
    app_state: State<'_, AppState>,
    item_service: State<'_, Arc<ItemService>>,
) -> Result<Option<LocalUser>, String> {
    match user_repo.get_active_user().await {
        Ok(Some(user)) => {
            // Set the AppState to this user
            app_state.set_user(user.id.clone(), user.username.clone()).await;
            
            // Update last_login timestamp
            user_repo.update_last_login(&user.id)
                .await
                .map_err(|e| format!("Failed to update last login: {}", e))?;
            
            // Auto-claim orphaned items
            match item_service.claim_offline_items(&user.id).await {
                Ok(count) => {
                    if count > 0 {
                        println!("✅ Migrated {} orphaned items", count);
                        
                        // Trigger background sync to push claimed items to Supabase
                        let item_service_bg = item_service.inner().clone();
                        let user_id_bg = user.id.clone();
                        tokio::spawn(async move {
                            match item_service_bg.sync_local_to_remote(&user_id_bg).await {
                                Ok(synced_count) => {
                                    println!("⬆️ Synced {} local items to remote after migration", synced_count);
                                }
                                Err(_) => {
                                    eprintln!("⚠️ Failed to sync local items to remote after migration");
                                }
                            }
                        });
                    } else {
                        println!("✓ No orphaned items found");
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to claim orphaned items: {}", e);
                }
            }
            
            Ok(Some(user))
        }
        Ok(None) => {
            app_state.clear_user_id().await;
            Ok(None)
        },
        Err(e) => Err(format!("Failed to check for active user: {}", e))
    }
}
