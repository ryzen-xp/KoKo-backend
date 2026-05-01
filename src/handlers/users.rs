use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
};

use crate::errors::AppError;
use crate::models::user::{CreateUserDto, User, UserResponse};
use crate::AppState;

// [Post] create_user

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(user): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    if user.username.trim().is_empty() {
        return Err(AppError::Validation("username is required".to_string()));
    }

    if user.handle.trim().is_empty() {
        return Err(AppError::Validation("handle is required".to_string()));
    }

    if user.email.trim().is_empty() {
        return Err(AppError::Validation("email is required".to_string()));
    }

    if user.password.trim().is_empty() {
        return Err(AppError::Validation("password is required".to_string()));
    }

    let created_user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, handle, email, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            username,
            handle,
            email,
            password_hash,
            bio,
            avatar_url,
            banner_url,
            is_verified,
            is_private,
            follower_count,
            following_count,
            tweet_count,
            created_at,
            updated_at
        "#,
    )
    .bind(user.username.trim())
    .bind(user.handle.trim())
    .bind(user.email.trim())
    .bind(user.password.trim())
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(created_user.into())))
}

//  [Patch]  update users

//  [Get]  get_users

//  [Delete]  remove_userss
