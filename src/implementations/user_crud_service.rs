use crate::models::user_model::{UserDao, UserDto};
use sqlx::PgPool;

// pub async fn create_user(
//     pool: &PgPool,
//     username: &str,
//     password_hash: &str,
//     plan_limit: i32,
//     api_key: &str,
// ) -> Result<(), sqlx::Error> {
//     sqlx::query(
//         r#"
//         INSERT INTO users (id, username, password_hash, api_key, plan_limit, used_count)
//         VALUES ($1, $2, $3, $4, $5, 0)
//         "#,
//     )
//     .bind(Uuid::new_v4())
//     .bind(username)
//     .bind(password_hash)
//     .bind(api_key)
//     .bind(plan_limit)
//     .execute(pool)
//     .await?;

//     Ok(())
// }

// pub async fn get_user_by_api_key(
//     pool: &PgPool,
//     api_key: &str,
// ) -> Result<Option<UserDto>, sqlx::Error> {
//     let user = sqlx::query_as::<_, UserDto>(
//         r#"
//         SELECT id, username, password_hash, api_key, plan_limit, used_count, created_at
//         FROM users
//         WHERE api_key = $1
//         "#,
//     )
//     .bind(api_key)
//     .fetch_optional(pool)
//     .await?;

//     Ok(user)
// }

pub async fn increment_usage_safe(pool: &PgPool, api_key: &str) -> Result<UserDto, sqlx::Error> {
    sqlx::query_as::<_, UserDto>(
        r#"
        UPDATE email_user_tbl
        SET used_count = used_count + 1
        WHERE api_key = $1
          AND used_count < plan_limit
        RETURNING username, plan_limit, used_count
        "#,
    )
    .bind(api_key)
    .fetch_one(pool)
    .await
}

pub async fn get_user(pool: &PgPool, api_key: &str) -> Result<UserDto, sqlx::Error> {
    sqlx::query_as::<_, UserDto>(
        r#"
        SELECT username, plan_limit, used_count
        FROM email_user_tbl
        WHERE api_key = $1
        "#,
    )
    .bind(api_key)
    .fetch_one(pool)
    .await
}

pub async fn get_all_users_with_limit(
    pool: &PgPool,
    limit: &i64,
) -> Result<Vec<UserDao>, sqlx::Error> {
    sqlx::query_as::<_, UserDao>(
        r#"
        SELECT *
FROM email_user_tbl
LIMIT $1;
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}