use db_service::{
    pagination::PageInfo as DbPageInfo,
    schema::{
        market::Market as DbMarket, user_holdings::UserIdWithShares, user_trades::MarketTrades,
    },
};
use utility_helpers::to_f64_verbose;

use crate::{
    generated::{
        common::PageInfo,
        markets::{
            GetMarketBookResponse, Market, MarketTrade as GeneratedMarketTrade, OrderBook,
            OrderLevel, UserWithTotalHoldings,
        },
    },
    utils::clickhouse_schema::GetOrderBook,
};
use crate::{ state::SafeState,};

pub mod market_services;
pub mod price_services;

// all type conversations.....

pub async fn from_db_market(
    value: &DbMarket,
    yes_price: f32,
    no_price: f32,
    pg_pool: &SafeState,
) -> Market {
    // Base parent/child market
    let mut market = Market {
        id: value.id.to_string(),
        name: value.name.clone(),
        description: value.description.clone(),
        logo: value.logo.clone(),
        status: value.status as i32,
        liquidity_b: to_f64_verbose(value.liquidity_b),
        final_outcome: value.final_outcome as i32,
        created_at: value.created_at.to_string(),
        updated_at: value.updated_at.to_string(),
        market_expiry: value.market_expiry.to_string(),
        yes_price,
        no_price,

        // New fields
        parent_id: value.parent_id.map(|id| id.to_string()).unwrap_or_default(),
        is_event: value.is_event,
        child_markets: vec![],
        category: value.category.clone().unwrap_or_default(),
        resolution_criteria: value.resolution_criteria.clone().unwrap_or_default(),
        slug: value.slug.clone().unwrap_or_default(),
    };

    // If this market is a parent and has children, fetch them recursively
    if value.is_event {
        if let Some(child_ids) = &value.child_market_ids {
            let mut children_proto = vec![];
            for child_id in child_ids {
                if let Ok(Some(child_db)) =
                    &DbMarket::get_market_by_id(&pg_pool.db_pool, child_id).await
                {
                    // Recursive conversion for child market
                    let child_proto = from_db_market_child(&child_db, 0.5, 0.5,).await;
                    children_proto.push(child_proto);
                }
            }
            market.child_markets = children_proto;
        }
    }

    market
}


pub async fn from_db_market_child(
    value: &DbMarket,
    yes_price: f32,
    no_price: f32,

) -> Market {
    // Base parent/child market
    let mut market = Market {
        id: value.id.to_string(),
        name: value.name.clone(),
        description: value.description.clone(),
        logo: value.logo.clone(),
        status: value.status as i32,
        liquidity_b: to_f64_verbose(value.liquidity_b),
        final_outcome: value.final_outcome as i32,
        created_at: value.created_at.to_string(),
        updated_at: value.updated_at.to_string(),
        market_expiry: value.market_expiry.to_string(),
        yes_price,
        no_price,

        // New fields
         parent_id: value.parent_id.map(|id| id.to_string()).unwrap_or_default(),
        is_event: value.is_event,
        child_markets: vec![],
        category: value.category.clone().unwrap_or_default(),
        resolution_criteria: value.resolution_criteria.clone().unwrap_or_default(),
        slug: value.slug.clone().unwrap_or_default(),
    };

   market
}



impl From<DbPageInfo> for PageInfo {
    fn from(value: DbPageInfo) -> Self {
        PageInfo {
            page: value.page,
            page_size: value.page_size,
            total_items: value.total_items,
            total_pages: value.total_pages,
        }
    }
}
impl From<MarketTrades> for GeneratedMarketTrade {
    fn from(value: MarketTrades) -> Self {
        GeneratedMarketTrade {
            created_at: value.timestamp.to_string(),
            email: value.email,
            avatar: value.avatar,
            id: value.id.to_string(),
            name: value.name,
            outcome: value.outcome as i32,
            price: to_f64_verbose(value.price),
            quantity: to_f64_verbose(value.quantity),
            trade_type: value.trade_type as i32,
        }
    }
}

impl From<UserIdWithShares> for UserWithTotalHoldings {
    fn from(value: UserIdWithShares) -> Self {
        UserWithTotalHoldings {
            user_id: value.user_id.to_string(),
            total_shares: value
                .total_shares
                .map_or_else(|| 0.0, |shares| to_f64_verbose(shares)),
            total_yes_shares: value
                .total_yes_shares
                .map_or_else(|| 0.0, |shares| to_f64_verbose(shares)),
            total_no_shares: value
                .total_no_shares
                .map_or_else(|| 0.0, |shares| to_f64_verbose(shares)),
            username: value.username.unwrap_or_default(),
            avatar: value.avatar.unwrap_or_default(),
        }
    }
}

pub fn to_resp_for_market_book(data: GetOrderBook) -> GetMarketBookResponse {
    GetMarketBookResponse {
        market_id: data.market_id.to_string(),
        yes_book: Some(OrderBook {
            bids: data
                .yes_bids
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
            asks: data
                .yes_asks
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
        }),
        no_book: Some(OrderBook {
            bids: data
                .no_bids
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
            asks: data
                .no_asks
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
        }),
    }
}
