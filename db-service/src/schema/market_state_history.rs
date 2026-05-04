use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Default, Serialize)]
pub struct MarketStateHistory {
    pub id: Uuid,
    pub market_id: Uuid,
    pub q_yes: Decimal,
    pub q_no: Decimal,
    pub liquidity_b: Decimal,
    pub price_yes: Decimal,
    pub price_no: Decimal,
    pub created_at: NaiveDateTime,
}



impl MarketStateHistory {
    pub async fn create_market_state_snapshot(
        pg_pool: &sqlx::PgPool,
        market_id: Uuid,
        q_yes: Decimal,
        q_no: Decimal,
        liquidity_b: Decimal,
        price_yes: Decimal,
        price_no: Decimal,
    ) -> Result<MarketStateHistory, sqlx::Error> {

     
    let snapshot = sqlx::query_as::<_, MarketStateHistory>(
    r#"
    INSERT INTO polymarket.market_state_history (
        market_id,
        q_yes,
        q_no,
        liquidity_b,
        price_yes,
        price_no
    )
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING
        id,
        market_id,
        q_yes,
        q_no,
        liquidity_b,
        price_yes,
        price_no,
        created_at
    "#
)
.bind(market_id)
.bind(q_yes)
.bind(q_no)
.bind(liquidity_b)
.bind(price_yes)
.bind(price_no)
.fetch_one(pg_pool)
.await?;

        Ok(snapshot)
    }


    pub async fn get_market_state_history_by_interval(
        pg_pool: &sqlx::PgPool,
        market_id: Uuid,
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    ) -> Result<Vec<MarketStateHistory>, sqlx::Error> {

        let history = sqlx::query_as!(
            MarketStateHistory,
            r#"
            SELECT
                id,
                market_id,
                q_yes,
                q_no,
                liquidity_b,
                price_yes,
                price_no,
                created_at
            FROM polymarket.market_state_history
            WHERE market_id = $1
              AND created_at BETWEEN $2 AND $3
            ORDER BY created_at ASC
            "#,
            market_id,
            start_time,
            end_time
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(history)
    }


// In db-service/src/schema/market_state_history.rs
pub async fn get_latest_market_state(
    pool:  &sqlx::PgPool,
    market_id: Uuid,
) -> Result<Option<MarketStateHistory>, sqlx::Error> {
    let record = sqlx::query_as!(
        MarketStateHistory,
        r#"
        SELECT *
        FROM polymarket.market_state_history
        WHERE market_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        market_id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(record)
}




}


