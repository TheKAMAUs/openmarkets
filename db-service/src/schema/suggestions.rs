use sqlx::prelude::FromRow;
use sqlx::PgPool;  // ✅ Added missing import
use uuid::Uuid;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::schema::enums::SuggestionStatus;  // ✅ Import the enum

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MarketSuggestion {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub upvotes: i32,
    pub status: SuggestionStatus,  // ✅ Use enum, not String
    pub admin_notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// For creating a new suggestion
#[derive(Debug, Deserialize)]
pub struct CreateSuggestionRequest {
    pub title: String,
    pub description: String,
    pub category: Option<String>,
}  // ✅ Only ONE definition

// For API response with user info
#[derive(Debug, Serialize)]
pub struct SuggestionResponse {
    pub id: Uuid,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub upvotes: i32,
    pub status: SuggestionStatus,  // ✅ Use enum
    pub admin_notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub user_voted: bool,
}

// For updating status (if needed)
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: SuggestionStatus,
    pub admin_notes: Option<String>,
}

pub struct SuggestionActions;

impl SuggestionActions {
/// Create a suggestion
pub async fn create(
    pool: &PgPool,
    req: CreateSuggestionRequest,
    user_id: Uuid,
) -> Result<MarketSuggestion, sqlx::Error> {
    sqlx::query_as!(
        MarketSuggestion,
        r#"
        INSERT INTO polymarket.market_suggestions (
            user_id, title, description, category, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        RETURNING 
            id, user_id, title, description, category, upvotes,
            status as "status: SuggestionStatus",  -- ✅ Cast to enum
            admin_notes, created_at, updated_at
        "#,
        user_id,
        req.title,
        req.description,
        req.category as Option<String>, 
    )
    .fetch_one(pool)
    .await
}

/// Get all suggestions with user vote status
pub async fn get_all(
    pool: &PgPool,
    current_user_id: Option<Uuid>,
) -> Result<Vec<SuggestionResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            s.id,
            s.title,
            s.description,
            s.category,
            s.upvotes,
            s.status as "status: SuggestionStatus",  -- ✅ Cast to enum
            s.admin_notes,
            s.created_at,
            s.updated_at,
            u.name as user_name,
            u.avatar as user_avatar,
            EXISTS(
                SELECT 1 FROM polymarket.suggestion_votes sv 
                WHERE sv.suggestion_id = s.id AND sv.user_id = $1
            ) as "user_voted!"
        FROM polymarket.market_suggestions s
        JOIN polymarket.users u ON u.id = s.user_id
        ORDER BY 
            CASE WHEN s.status::text = 'pending' THEN 0 ELSE 1 END,  -- ✅ Compare as text
            s.upvotes DESC,
            s.created_at DESC
        "#,
        current_user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| SuggestionResponse {
        id: r.id,
        user_name: r.user_name,
        user_avatar: r.user_avatar,
        title: r.title,
        description: r.description,
        category: r.category,
        upvotes: r.upvotes,
        status: r.status,  // ✅ Now this is SuggestionStatus enum
        admin_notes: r.admin_notes,
        created_at: r.created_at,
        updated_at: r.updated_at,
        user_voted: r.user_voted,
    }).collect())
}

    /// Upvote a suggestion
    pub async fn upvote(
        pool: &PgPool,
        suggestion_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Try to insert vote
        let result = sqlx::query!(
            r#"
            INSERT INTO polymarket.suggestion_votes (suggestion_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            suggestion_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // If vote was inserted, increment upvotes
        if result.rows_affected() > 0 {
            sqlx::query!(
                r#"
                UPDATE polymarket.market_suggestions 
                SET upvotes = upvotes + 1 
                WHERE id = $1
                "#,
                suggestion_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await
    }

    /// Remove upvote
    pub async fn remove_upvote(
        pool: &PgPool,
        suggestion_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        let result = sqlx::query!(
            r#"
            DELETE FROM polymarket.suggestion_votes
            WHERE suggestion_id = $1 AND user_id = $2
            "#,
            suggestion_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            sqlx::query!(
                r#"
                UPDATE polymarket.market_suggestions 
                SET upvotes = upvotes - 1 
                WHERE id = $1
                "#,
                suggestion_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await
    }

     /// Update status with admin notes
/// Update status with admin notes
pub async fn update_status(
    pool: &PgPool,
    suggestion_id: Uuid,
    _admin_id: Uuid,  // ✅ Prefix with underscore if unused
    status: SuggestionStatus,  // ✅ Use enum, not &str
    admin_notes: Option<String>,
) -> Result<MarketSuggestion, sqlx::Error> {
    sqlx::query_as!(
        MarketSuggestion,
        r#"
        UPDATE polymarket.market_suggestions 
        SET 
            status = $1,
            admin_notes = COALESCE($2, admin_notes),
            updated_at = NOW()
        WHERE id = $3
        RETURNING 
            id, user_id, title, description, category, upvotes,
            status as "status: SuggestionStatus",  -- ✅ Cast to enum
            admin_notes, created_at, updated_at
        "#,
        status as SuggestionStatus,  
        admin_notes,
        suggestion_id
    )
    .fetch_one(pool)
    .await
}

}