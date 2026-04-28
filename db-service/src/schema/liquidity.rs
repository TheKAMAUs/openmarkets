use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use rust_decimal::Decimal;
use uuid::Uuid;
use sqlx::Executor;
use sqlx::Postgres;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LpPosition {
    pub lp_position_id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub amount_deposited: Decimal,
    pub shares_of_pool: Decimal,
    pub total_fees_earned: Decimal,
    pub withdrawn_amount: Decimal,
    pub is_active: bool,
    pub created_at: NaiveDateTime,  // Changed
    pub updated_at: NaiveDateTime,  // Changed
}

impl LpPosition {
pub async fn add_liquidity(
    pool: &PgPool,
    user_id: Uuid,
    market_id: Uuid,
    amount: Decimal,
    shares: Decimal,
) -> Result<LpPosition, sqlx::Error> {
    let rec = sqlx::query_as::<_, LpPosition>(
        r#"
        INSERT INTO polymarket.lp_positions (user_id, market_id, amount_deposited, shares_of_pool)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(market_id)
    .bind(amount)
    .bind(shares)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}

pub async fn remove_liquidity(
    pool: &PgPool,
    lp_position_id: Uuid,
    withdraw_amount: Decimal,
) -> Result<LpPosition, sqlx::Error> {
    let rec = sqlx::query_as::<_, LpPosition>(
        r#"
        UPDATE polymarket.lp_positions
        SET withdrawn_amount = withdrawn_amount + $1,
            amount_deposited = amount_deposited - $1,
            updated_at = NOW(),
            is_active = CASE
                WHEN amount_deposited - $1 <= 0 THEN FALSE
                ELSE TRUE
            END
        WHERE lp_position_id = $2
        AND amount_deposited >= $1
        RETURNING *
        "#,
    )
    .bind(withdraw_amount)
    .bind(lp_position_id)
    .fetch_one(pool)
    .await?;

    Ok(rec)
}


 /// Get all active LP positions for a specific market
pub async fn get_lps_by_market(
    executor: impl Executor<'_, Database = Postgres>,
    market_id: Uuid,
) -> Result<Vec<LpPosition>, sqlx::Error> {
    let lps = sqlx::query_as::<_, LpPosition>(
        r#"
        SELECT 
            lp_position_id,
            user_id,
            market_id,
            amount_deposited,
            shares_of_pool,
            total_fees_earned,
            withdrawn_amount,
            is_active,
            created_at,
            updated_at
        FROM polymarket.lp_positions
        WHERE market_id = $1 
            AND is_active = true
        ORDER BY created_at ASC
        "#,
    )
    .bind(market_id)
    .fetch_all(executor)
    .await?;
    
    Ok(lps)
}
    




    pub async fn get_fee_earnings(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Decimal, sqlx::Error> {
        let row: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(total_fees_earned), 0) FROM polymarket.lp_positions WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(row.0)
    }

    pub async fn lp_is_eligible_for_fees(
        pool: &PgPool,
        lp_position_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let row: (bool,) = sqlx::query_as(
            r#"
            SELECT created_at <= NOW() - INTERVAL '24 hours'
            FROM polymarket.lp_positions
            WHERE lp_position_id = $1
            "#
        )
        .bind(lp_position_id)
        .fetch_one(pool)
        .await?;

        Ok(row.0)
    }

    pub async fn distribute_fees_to_lps(
        pool: &PgPool,
        market_id: Uuid,
        total_fee: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE polymarket.lp_positions
            SET total_fees_earned = total_fees_earned + ($2 * (shares_of_pool /
                (SELECT SUM(shares_of_pool) FROM lp_positions WHERE market_id = $1)))
            WHERE market_id = $1
            AND created_at <= NOW() - INTERVAL '24 hours'
            "#
        )
        .bind(market_id)
        .bind(total_fee)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_lp_position(
        pool: &PgPool,
        lp_position_id: Uuid,
    ) -> Result<Option<LpPosition>, sqlx::Error> {
        let rec = sqlx::query_as::<_, LpPosition>(
            r#"
            SELECT * FROM polymarket.lp_positions WHERE lp_position_id = $1
            "#
        )
        .bind(lp_position_id)
        .fetch_optional(pool)
        .await?;

        Ok(rec)
    }

    pub async fn get_lp_positions_by_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<LpPosition>, sqlx::Error> {
        let recs = sqlx::query_as::<_, LpPosition>(
            r#"
            SELECT * FROM polymarket.lp_positions WHERE user_id = $1 AND is_active = true
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(recs)
    }

    pub async fn get_lp_positions_by_market(
        pool: &PgPool,
        market_id: Uuid,
    ) -> Result<Vec<LpPosition>, sqlx::Error> {
        let recs = sqlx::query_as::<_, LpPosition>(
            r#"
            SELECT * FROM polymarket.lp_positions WHERE market_id = $1 AND is_active = true
            "#
        )
        .bind(market_id)
        .fetch_all(pool)
        .await?;

        Ok(recs)
    }

pub async fn update_fees_earned_and_shares(
    pool: &PgPool,
    lp_position_id: Uuid,
    fee_amount: Decimal,
    share_percentage: Decimal,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE polymarket.lp_positions
        SET total_fees_earned = total_fees_earned + $2,
            shares_of_pool = $3,
            updated_at = NOW()
        WHERE lp_position_id = $1
        "#
    )
    .bind(lp_position_id)
    .bind(fee_amount)
    .bind(share_percentage)
    .execute(pool)
    .await?;

    Ok(())
}


/// Get total pool shares for a specific market
pub async fn get_total_pool_shares(
    executor: impl Executor<'_, Database = Postgres>,
    market_id: Uuid,
) -> Result<Decimal, sqlx::Error> {
    let total_shares = sqlx::query_scalar!(
        r#"
        SELECT SUM(shares_of_pool) FROM polymarket.lp_positions 
        WHERE market_id = $1 
            AND is_active = true
        "#,
        market_id,
    )
    .fetch_one(executor)
    .await?
    .unwrap_or(Decimal::ZERO);

    Ok(total_shares)
}


pub async fn get_total_amount_deposited(
    pool: &PgPool,
    market_id: Uuid,
) -> Result<Decimal, sqlx::Error> {
    let row: (Decimal,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(amount_deposited), 0) FROM polymarket.lp_positions 
        WHERE market_id = $1 AND is_active = true
        "#
    )
    .bind(market_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}



}


