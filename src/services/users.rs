use std::sync::Arc;

use crate::{
    error::ApiError,
    models::user::{Role, User},
    routes::auth::AuthPayload,
    services::auth::{hash_string, AuthError},
    ApiState,
};

pub async fn create_user(
    state: &Arc<ApiState>,
    payload: AuthPayload,
    roles: &[Role],
) -> Result<User, ApiError> {
    if state
        .db
        .users()
        .find_by_username(&payload.username)
        .await?
        .is_some()
    {
        return Err(AuthError::UsernameAlreadyTaken.into());
    }

    let new_user = User::new(&payload.username, &hash_string(&payload.password), roles);
    let created_user = state.db.users().create(new_user).await?;

    Ok(created_user)
}
