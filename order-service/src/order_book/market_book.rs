use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};

use std::str::FromStr;
 use rust_decimal::MathematicalOps; 
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use utility_helpers::{ log_error, log_info,log_warn};

use crate::order_book::outcome_book::OrderBookMatchedOutput;
use db_service::schema::liquidity::LpPosition;

use super::outcome_book::OutcomeBook;

// #[derive(Debug)]
// pub(crate) struct MarketBook {
//     yes_order_book: OutcomeBook,
//     no_order_book: OutcomeBook,

//     pub(crate) executed_yes_buy_volume: Decimal,
//     pub(crate) executed_no_buy_volume: Decimal,

//     pub(crate) current_yes_price: Decimal,
//     pub(crate) current_no_price: Decimal,

//     /// Liquidity parameter of the market
//     ///
//     /// The higher `b` = more liquidity, slower price changes
//     pub(crate) liquidity_b: Decimal,
// }


#[derive(Debug)]
pub(crate) struct MarketBook {

    yes_order_book: OutcomeBook,
    no_order_book: OutcomeBook,
    // Volume tracking (keep)
    pub(crate) executed_yes_buy_volume: Decimal,
    pub(crate) executed_no_buy_volume: Decimal,
    
    // Current prices (keep)
    pub(crate) current_yes_price: Decimal,
    pub(crate) current_no_price: Decimal,
    
    // LMSR parameter (keep)
    pub(crate) liquidity_b: Decimal,
    
    // NEW: LMSR state variables
    pub(crate) q_yes: Decimal,    // Net number of Yes shares sold to traders
    pub(crate) q_no: Decimal,     // Net number of No shares sold to traders
}




pub struct LmsrExecutionResult {
    pub order_id: Uuid,
    pub shares_bought: Decimal,      // How many shares the user received
    pub total_cost: Decimal,          // Total cost in KES
    pub average_price: Decimal,       // Average execution price
    pub new_yes_price: Decimal,       // Updated market price after trade
    pub new_no_price: Decimal,        // Updated market price after trade
    pub shares_after_qty: Decimal,    // New q_y or q_n in the LMSR state
    pub q_yes: Decimal,               // Full LMSR state after trade
    pub q_no: Decimal,
    pub liquidity_b: Decimal,         // Market liquidity parameter (unchanged)
    pub order_side: OrderSide,        // BUY or SELL side of the order
}


impl MarketBook {
      pub(super) fn new(liquidity_b: Decimal, q_yes: Decimal, q_no: Decimal) -> Self {
        Self {

  yes_order_book: OutcomeBook::default(),
            no_order_book: OutcomeBook::default(),

            executed_yes_buy_volume: Decimal::ZERO,
            executed_no_buy_volume: Decimal::ZERO,
            current_yes_price: Decimal::new(5, 1),
            current_no_price: Decimal::new(5, 1),
            liquidity_b,
            q_yes,
            q_no,
        }
    }

    /// Entry point — routes an incoming order through LMSR and returns results.
    /// Replaces the old LOB-based process_order entirely.
    pub(super) fn process_order(
        &mut self,
        order: &mut Order,
    ) -> Vec<LmsrExecutionResult> {
        if order.quantity <= Decimal::ZERO || order.price <= Decimal::ZERO {
            log_warn!(
                "Rejecting order {} — invalid quantity ({}) or price ({})",
                order.id, order.quantity, order.price
            );
            order.status = OrderStatus::CANCELLED;
            return Vec::new();
        }

        let results = self.create_market_order(order, order.quantity);

        // If LMSR couldn't fill (e.g. cost overflow), ensure status is consistent
        if results.is_empty() && order.status != OrderStatus::CANCELLED {
            order.status = OrderStatus::CANCELLED;
        }

        results
    }



// Helper function to calculate LMSR trade for a given order
// Helper function to calculate LMSR trade for a given order
pub(super) fn create_market_order(
    &mut self,
    order: &mut Order,
    budget: Decimal,
) -> Vec<LmsrExecutionResult> {
    println!("🔵 [DEBUG] ========================================");
    println!("🔵 [DEBUG] create_market_order ENTERED");
    println!("🔵 [DEBUG] Order ID: {:?}", order.id);
    println!("🔵 [DEBUG] Order side: {:?}", order.side);
    println!("🔵 [DEBUG] Order outcome: {:?}", order.outcome);
    println!("🔵 [DEBUG] Order quantity: {}", order.quantity);
    println!("🔵 [DEBUG] Order price: {}", order.price);
    println!("🔵 [DEBUG] Budget param: {}", budget);
    println!("🔵 [DEBUG] Current self.q_yes: {}", self.q_yes);
    println!("🔵 [DEBUG] Current self.q_no: {}", self.q_no);
    println!("🔵 [DEBUG] Current self.liquidity_b: {}", self.liquidity_b);
    println!("🔵 [DEBUG] Current self.current_yes_price: {}", self.current_yes_price);
    println!("🔵 [DEBUG] Current self.current_no_price: {}", self.current_no_price);
    
    let mut results = Vec::new();
    
    let amount_spent = order.quantity;
    let execution_price = order.price;
    
    println!("🔵 [DEBUG] amount_spent: {}, execution_price: {}", amount_spent, execution_price);
    
    if amount_spent <= Decimal::ZERO || execution_price <= Decimal::ZERO {
        println!("⚠️ [DEBUG] INVALID ORDER - amount_spent or execution_price is ZERO");
        log_warn!("Invalid order - amount_spent: {}, execution_price: {}", amount_spent, execution_price);
        order.status = OrderStatus::CANCELLED;
        return results;
    }
    
    println!("🔵 [DEBUG] Validation passed, calling calculate_lmsr_trade_for_order...");
    
    // Calculate shares and update market
    let (shares_received, actual_cost, avg_price, new_yes_price, new_no_price, new_q) = 
        self.calculate_lmsr_trade_for_order(order, order.side);
    
    println!("🔵 [DEBUG] calculate_lmsr_trade_for_order returned:");
    println!("🔵 [DEBUG]   shares_received: {}", shares_received);
    println!("🔵 [DEBUG]   actual_cost: {}", actual_cost);
    println!("🔵 [DEBUG]   avg_price: {}", avg_price);
    println!("🔵 [DEBUG]   new_yes_price: {}", new_yes_price);
    println!("🔵 [DEBUG]   new_no_price: {}", new_no_price);
    println!("🔵 [DEBUG]   new_q: {}", new_q);
    
    // Since we used the exact price, actual_cost should equal amount_spent
    if shares_received > Decimal::ZERO {
        println!("✅ [DEBUG] Trade will be EXECUTED");
        order.filled_quantity = shares_received;
        order.quantity = amount_spent;
        order.price = avg_price;
        order.status = OrderStatus::FILLED;
        
        println!("🔵 [DEBUG] Order updated - filled_quantity: {}, status: {:?}", order.filled_quantity, order.status);
        
        // Track volume
        if order.side == OrderSide::BUY {
            match order.outcome {
                Outcome::YES => {
                    self.executed_yes_buy_volume += actual_cost;
                    println!("🔵 [DEBUG] Updated executed_yes_buy_volume: {}", self.executed_yes_buy_volume);
                },
                Outcome::NO => {
                    self.executed_no_buy_volume += actual_cost;
                    println!("🔵 [DEBUG] Updated executed_no_buy_volume: {}", self.executed_no_buy_volume);
                },
                _ => {}
            }
        }
        
        let result = LmsrExecutionResult {
            order_id: order.id,
            shares_bought: shares_received,
            total_cost: actual_cost,
            average_price: avg_price,
            new_yes_price,
            new_no_price,
            shares_after_qty: new_q,
            q_yes: self.q_yes,
            q_no: self.q_no,
            liquidity_b: self.liquidity_b,
            order_side: order.side,
        };
        results.push(result);
        
        println!("✅ [DEBUG] LmsrExecutionResult created and pushed to results");
        
        log_info!(
            "✅ Trade executed - Shares: {}, Cost: {}, Price: {}",
            shares_received, actual_cost, avg_price
        );
    } else {
        println!("❌ [DEBUG] Trade FAILED - shares_received is ZERO or NEGATIVE");
        order.status = OrderStatus::CANCELLED;
        log_warn!(
            "❌ Trade failed - shares_received: {}, amount_spent: {}",
            shares_received, amount_spent
        );
    }
    
    println!("🔵 [DEBUG] Calling update_market_price()");
    self.update_market_price();
    
    println!("🔵 [DEBUG] After update_market_price - yes: {}, no: {}", self.current_yes_price, self.current_no_price);
    println!("🔵 [DEBUG] Results count: {}", results.len());
    println!("🔵 [DEBUG] ========================================");
    
    results
}


// Helper function to calculate LMSR trade for a given order
pub(super) fn calculate_lmsr_trade_for_order(
    &mut self,
    order: &Order,
    side: OrderSide,
) -> (Decimal, Decimal, Decimal, Decimal, Decimal, Decimal) {
    if self.liquidity_b <= Decimal::ZERO {
        log_warn!("⚠️ LMSR trade failed - liquidity_b is zero or negative");
        return (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, 
                self.current_yes_price, self.current_no_price, Decimal::ZERO);
    }
    
    let b = self.liquidity_b;
    let is_yes = matches!(order.outcome, Outcome::YES);
    
    // Get current state
    let q_yes_orig = self.q_yes;
    let q_no_orig = self.q_no;
    
    // Simple calculation based on order fields
    let amount = order.quantity;      // Budget in KES
    let price = order.price;           // Execution price from LMSR
    
    // Calculate shares (amount / price)
    let shares = if price > Decimal::ZERO {
        amount / price
    } else {
        Decimal::ZERO
    };
    
    let actual_cost = amount;  // Simple - cost equals the budget
    let avg_price = price;
    
    log_info!(
        "📊 LMSR trade - Amount: {}, Price: {}, Shares: {}",
        amount, price, shares
    );
    
    // Update quantities based on trade
    let (new_q_yes, new_q_no) = match (side, is_yes) {
        (OrderSide::BUY, true) => (q_yes_orig + shares, q_no_orig),
        (OrderSide::BUY, false) => (q_yes_orig, q_no_orig + shares),
        (OrderSide::SELL, true) => (q_yes_orig - shares, q_no_orig),
        (OrderSide::SELL, false) => (q_yes_orig, q_no_orig - shares),
    };
    
    // Ensure non-negative
    let new_q_yes = new_q_yes.max(Decimal::ZERO);
    let new_q_no = new_q_no.max(Decimal::ZERO);
    
    // Use LMSR to calculate new prices based on updated quantities
    let new_exp_yes = (new_q_yes / b).exp();
    let new_exp_no = (new_q_no / b).exp();
    let total_exp = new_exp_yes + new_exp_no;
    
    let new_yes_price = if total_exp > Decimal::ZERO {
        new_exp_yes / total_exp
    } else {
        dec!(0.5)
    };
    let new_no_price = if total_exp > Decimal::ZERO {
        new_exp_no / total_exp
    } else {
        dec!(0.5)
    };
    
    // Update state
    self.q_yes = new_q_yes;
    self.q_no = new_q_no;
    self.current_yes_price = new_yes_price;
    self.current_no_price = new_no_price;
    
    // Get the new q value for the traded outcome
    let new_q = if is_yes { new_q_yes } else { new_q_no };
    
    log_info!(
        "✅ LMSR trade result - shares: {}, actual_cost: {}, new_yes_price: {}, new_no_price: {}",
        shares, actual_cost, new_yes_price, new_no_price
    );
    
    (shares, actual_cost, avg_price, new_yes_price, new_no_price, new_q)
}



pub fn calculate_price(&self, outcome: Outcome) -> Decimal {
    if self.liquidity_b <= Decimal::ZERO {
        return dec!(0.5);
    }
    
    let b = self.liquidity_b;
    let exp_yes = (self.q_yes / b).exp();
    let exp_no = (self.q_no / b).exp();
    let total = exp_yes + exp_no;
    
    if total <= Decimal::ZERO {
        return dec!(0.5);
    }
    
    match outcome {
        Outcome::YES => exp_yes / total,
        Outcome::NO => exp_no / total,
        _ => dec!(0.5),
    }
}

fn update_market_price(&mut self) {
    // Pure LMSR - prices derived from q_yes and q_no
    self.current_yes_price = self.calculate_price(Outcome::YES);
    self.current_no_price = self.calculate_price(Outcome::NO);
    
    log_info!(
        "LMSR - Yes: {:.6}, No: {:.6}, q_yes: {}, q_no: {}, b: {}",
        self.current_yes_price,
        self.current_no_price,
        self.q_yes,
        self.q_no,
        self.liquidity_b
    );
}



    pub(super) fn update_order(
        &mut self,
        order: &mut Order,
        new_quantity: Decimal,
        new_price: Decimal,
    ) -> bool {
        let result = match order.outcome {
            Outcome::YES => self
                .yes_order_book
                .update_order(order, new_price, new_quantity),
            Outcome::NO => self
                .no_order_book
                .update_order(order, new_price, new_quantity),
            _ => false,
        };
        if result {
            self.update_market_price();
        }
        result
    }

    pub(super) fn remove_order(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        outcome: Outcome,
        price: Decimal,
    ) -> bool {
        let result = match outcome {
            Outcome::YES => self.yes_order_book.remove_order(order_id, side, price),
            Outcome::NO => self.no_order_book.remove_order(order_id, side, price),
            _ => false,
        };

        if result {
            self.update_market_price();
        }

        result
    }

    pub(crate) fn get_order_book(&self, outcome: Outcome) -> Option<&OutcomeBook> {
        match outcome {
            Outcome::YES => Some(&self.yes_order_book),
            Outcome::NO => Some(&self.no_order_book),
            _ => None,
        }
    }









}
















// impl MarketBook {
//     pub(super) fn new(liquidity_b: Decimal) -> Self {
//         Self {
//             executed_yes_buy_volume: Decimal::ZERO,
//             executed_no_buy_volume: Decimal::ZERO,
//             current_yes_price: Decimal::new(5, 1),   // 0.5
//             current_no_price: Decimal::new(5, 1),    // 0.5
//             liquidity_b,
//             q_yes: Decimal::ZERO,
//             q_no: Decimal::ZERO,
//         }
//     }

//     pub(super) fn add_order(&mut self, order: &Order) {
//         match order.outcome {
//             Outcome::YES => self.yes_order_book.add_order(order),
//             Outcome::NO => self.no_order_book.add_order(order),
//             _ => {}
//         }
//         self.update_market_price();
//     }

//     pub(super) fn process_order(&mut self, order: &mut Order) -> Vec<OrderBookMatchedOutput> {
//         let matches = match order.outcome {
//             Outcome::YES => self.yes_order_book.match_order(order),
//             Outcome::NO => self.no_order_book.match_order(order),
//             _ => Vec::new(),
//         };

//         if order.status == OrderStatus::OPEN || order.status == OrderStatus::PendingUpdate {
//             self.add_order(order);
//         }
//         self.update_market_price();
//         matches
//     }

//     pub(super) fn create_market_order(
//         &mut self,
//         order: &mut Order,
//         budget: Decimal,
//     ) -> Vec<OrderBookMatchedOutput> {
//         let matches = match order.outcome {
//             Outcome::YES => self.yes_order_book.create_market_order(order, budget),
//             Outcome::NO => self.no_order_book.create_market_order(order, budget),
//             _ => Vec::new(),
//         };

//         if order.side == OrderSide::BUY && order.filled_quantity > Decimal::ZERO {
//             let executed_value = matches
//                 .iter()
//                 .map(|m| m.price * m.matched_quantity)
//                 .sum::<Decimal>();

//             match order.outcome {
//                 Outcome::YES => self.executed_yes_buy_volume += executed_value,
//                 Outcome::NO => self.executed_no_buy_volume += executed_value,
//                 _ => {}
//             }
//         }

//         self.update_market_price();
//         matches
//     }

//     pub(super) fn update_order(
//         &mut self,
//         order: &mut Order,
//         new_quantity: Decimal,
//         new_price: Decimal,
//     ) -> bool {
//         let result = match order.outcome {
//             Outcome::YES => self
//                 .yes_order_book
//                 .update_order(order, new_price, new_quantity),
//             Outcome::NO => self
//                 .no_order_book
//                 .update_order(order, new_price, new_quantity),
//             _ => false,
//         };
//         if result {
//             self.update_market_price();
//         }
//         result
//     }

//     pub(super) fn remove_order(
//         &mut self,
//         order_id: Uuid,
//         side: OrderSide,
//         outcome: Outcome,
//         price: Decimal,
//     ) -> bool {
//         let result = match outcome {
//             Outcome::YES => self.yes_order_book.remove_order(order_id, side, price),
//             Outcome::NO => self.no_order_book.remove_order(order_id, side, price),
//             _ => false,
//         };

//         if result {
//             self.update_market_price();
//         }

//         result
//     }

//     pub(crate) fn get_order_book(&self, outcome: Outcome) -> Option<&OutcomeBook> {
//         match outcome {
//             Outcome::YES => Some(&self.yes_order_book),
//             Outcome::NO => Some(&self.no_order_book),
//             _ => None,
//         }
//     }

//     ///// Helpers //////

//     fn update_market_price(&mut self) {
//         // https://www.cultivatelabs.com/crowdsourced-forecasting-guide/how-does-logarithmic-market-scoring-rule-lmsr-work
//         // Refer above blogpost for better understanding on LMSR (Logarithmic Market Scoring Rule) price mechanism for prediction markets
//         if self.liquidity_b > Decimal::ZERO {
//             let funds_yes = self.calculate_total_funds(Outcome::YES);
//             let funds_no = self.calculate_total_funds(Outcome::NO);

//             let total_liquidity = self.liquidity_b * dec!(2); // 2 * b for both sides
//             let total_funds = funds_yes + funds_no;

//             if total_funds > Decimal::ZERO {
//                 let yes_weight = (self.liquidity_b + funds_yes) / (total_liquidity + total_funds);
//                 let no_weight = (self.liquidity_b + funds_no) / (total_liquidity + total_funds);

//                 let total_weight = yes_weight + no_weight;
//                 self.current_yes_price = yes_weight / total_weight;
//                 self.current_no_price = no_weight / total_weight;
//             } else {
//                 self.current_yes_price = dec!(0.5);
//                 self.current_no_price = dec!(0.5);
//             }
//         } else {
//             let yes_mid = self.calculate_midpoint_price(&self.yes_order_book);
//             let no_mid = self.calculate_midpoint_price(&self.no_order_book);

//             match (yes_mid, no_mid) {
//                 (Some(yes_price), Some(no_price)) => {
//                     let total = yes_price + no_price;
//                     if total > Decimal::ZERO {
//                         self.current_yes_price = yes_price / total;
//                         self.current_no_price = no_price / total;
//                     } else {
//                         self.current_yes_price = dec!(0.5);
//                         self.current_no_price = dec!(0.5);
//                     }
//                 }

//                 (Some(yes_price), None) => {
//                     self.current_yes_price = yes_price.min(dec!(0.95)); // cap at 0.95
//                     self.current_no_price = dec!(1) - self.current_yes_price;
//                 }
//                 (None, Some(no_price)) => {
//                     self.current_no_price = no_price.min(dec!(0.95)); // cap at 0.95
//                     self.current_yes_price = dec!(1) - self.current_no_price;
//                 }
//                 (None, None) => {
//                     self.current_yes_price = dec!(0.5);
//                     self.current_no_price = dec!(0.5);
//                 }
//             }
//         }
//     }

//     fn calculate_total_funds(&self, outcome: Outcome) -> Decimal {
//         // iterating over bids, because buyers have put their money. sellers are putting stocks (not money, so funds = bids for this part)
//         let book_funds = match outcome {
//             Outcome::YES => self
//                 .yes_order_book
//                 .bids
//                 .iter()
//                 .map(|(p, price_level)| *p * price_level.total_quantity)
//                 .sum(),
//             Outcome::NO => self
//                 .no_order_book
//                 .bids
//                 .iter()
//                 .map(|(p, price_level)| *p * price_level.total_quantity)
//                 .sum(),
//             _ => Decimal::ZERO,
//         };

//         let executed_funds = match outcome {
//             Outcome::YES => self.executed_yes_buy_volume,
//             Outcome::NO => self.executed_no_buy_volume,
//             _ => Decimal::ZERO,
//         };

//         book_funds + executed_funds
//     }

//     fn calculate_midpoint_price(&self, order_book: &OutcomeBook) -> Option<Decimal> {
//         match (order_book.best_bid(), order_book.best_ask()) {
//             (Some(bid), Some(ask)) => Some((bid + ask) / dec!(2)),
//             (Some(bid), None) => Some(bid),
//             (None, Some(ask)) => Some(ask),
//             (None, None) => None,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDateTime;
    use db_service::schema::enums::OrderType;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }
    #[test]
    fn test_market_order_empty_book_behavior() {
        let mut market_book = MarketBook::new(dec!(100));
        let market_id = Uuid::new_v4();

        let mut market_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        let budget = dec!(100); // Large budget but empty book
        let matches = market_book.create_market_order(&mut market_order, budget);

        // Results of empty book matching:
        assert_eq!(matches.len(), 0); // No matches
        assert_eq!(market_order.quantity, dec!(0)); // No quantity
        assert_eq!(market_order.filled_quantity, dec!(0)); // Nothing filled
        assert_eq!(market_order.status, OrderStatus::CANCELLED); // Still "cancelled"

        // Prices remain at default
        assert_eq!(market_book.current_yes_price, dec!(0.5));
        assert_eq!(market_book.current_no_price, dec!(0.5));

        // No executed volume tracked
        assert_eq!(market_book.executed_yes_buy_volume, dec!(0));
    }

    #[test]
    fn test_create_market_book() {
        let liquidity_b = Decimal::new(100, 0); // 100 units of liquidity

        let market_book = MarketBook::new(liquidity_b);

        assert_eq!(market_book.liquidity_b, liquidity_b);
        assert_eq!(market_book.current_yes_price, Decimal::new(5, 1)); // 0.5
        assert_eq!(market_book.current_no_price, Decimal::new(5, 1)); // 0.5
        assert!(market_book.yes_order_book.bids.is_empty());
        assert!(market_book.no_order_book.bids.is_empty());
        assert!(market_book.yes_order_book.asks.is_empty());
        assert!(market_book.no_order_book.asks.is_empty());
    }

    #[test]
    fn test_add_order_and_price_update() {
        let order_1 = Order {
            id: get_random_uuid(),
            outcome: Outcome::YES,
            side: OrderSide::BUY,
            price: Decimal::new(5, 1),     // 0.5
            quantity: Decimal::new(10, 0), // 10 units
            status: OrderStatus::OPEN,
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            market_id: get_random_uuid(),
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let order_2 = Order {
            id: get_random_uuid(),
            outcome: Outcome::NO,
            side: OrderSide::BUY,
            price: Decimal::new(5, 1),     // 0.5
            quantity: Decimal::new(10, 0), // 10 units
            status: OrderStatus::OPEN,
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            market_id: get_random_uuid(),
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let liquidity_b = Decimal::new(100, 0);
        let mut market_book = MarketBook::new(liquidity_b); // 100 units of liquidity

        market_book.add_order(&order_1);
        market_book.add_order(&order_1);
        market_book.add_order(&order_2);
        market_book.add_order(&order_2);

        assert_eq!(market_book.yes_order_book.bids.len(), 1);
        assert!(market_book.yes_order_book.bids.contains_key(&order_1.price));
        assert_eq!(
            market_book
                .yes_order_book
                .bids
                .get(&order_1.price)
                .unwrap()
                .total_quantity,
            order_1.quantity * dec!(2)
        );
        assert_eq!(market_book.current_yes_price, Decimal::new(5, 1)); //  0.5
        assert_eq!(market_book.current_no_price, Decimal::new(5, 1)); // 0.5

        market_book.add_order(&order_2); // adding another order on NO side to skew the price

        assert_ne!(market_book.current_yes_price, Decimal::new(5, 1)); // != 0.5
        assert_ne!(market_book.current_no_price, Decimal::new(5, 1)); // != 0.5
    }

    #[test]
    fn test_process_order() {
        let mut buy_order_1_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order_1_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(5, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut buy_order_1_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::NO,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order_1_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::NO,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(5, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut market_book = MarketBook::new(dec!(100));

        market_book.process_order(&mut buy_order_1_no);
        market_book.process_order(&mut buy_order_1_yes);

        let match_1 = market_book.process_order(&mut sell_order_1_no);
        let match_2 = market_book.process_order(&mut sell_order_1_yes);

        assert_eq!(match_1.len(), 1);
        assert_eq!(match_2.len(), 1);
        assert_eq!(match_1.get(0).unwrap().order_id, sell_order_1_no.id);
        assert_eq!(match_2.get(0).unwrap().order_id, sell_order_1_yes.id);
        assert_eq!(match_1.get(0).unwrap().opposite_order_id, buy_order_1_no.id);
        assert_eq!(
            match_2.get(0).unwrap().opposite_order_id,
            buy_order_1_yes.id
        );
    }

    #[test]
    fn test_remove_order() {
        let id = get_random_uuid();
        let price = dec!(0.5);
        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id,
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            price, // 0.5
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut market_book = MarketBook::new(dec!(100));

        market_book.add_order(&order);

        assert_eq!(market_book.yes_order_book.bids.len(), 1);

        market_book.remove_order(id, OrderSide::BUY, Outcome::YES, price);

        assert_eq!(market_book.yes_order_book.bids.len(), 0);
    }

    #[test]
    fn test_partial_fill() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify partial fill
        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(sell_order.filled_quantity, dec!(5));
        assert_eq!(matches.len(), 1);

        // Check the buy order was partially filled
        let price_level = outcome_book.bids.get(&buy_order.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(5));
    }

    #[test]
    fn test_match_multiple_orders_at_same_price() {
        let market_id = get_random_uuid();

        // Create multiple buy orders at the same price
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_2 = Order {
            created_at: get_created_at()
                .checked_add_signed(chrono::Duration::seconds(1))
                .unwrap(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25), // Same price
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(8),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify time priority matching
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].opposite_order_id, buy_order_1.id); // First order matched first (time priority)
        assert_eq!(matches[1].opposite_order_id, buy_order_2.id);
        assert_eq!(sell_order.filled_quantity, dec!(8));
        assert_eq!(sell_order.status, OrderStatus::FILLED);

        // Check remaining quantity in order book
        let price_level = outcome_book.bids.get(&buy_order_1.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(2)); // 10 - 8 = 2 remaining
    }

    #[test]
    fn test_match_order_zero_quantity() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(0), // Zero quantity
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify no matches for zero quantity
        assert_eq!(matches.len(), 0);
        assert_eq!(sell_order.filled_quantity, dec!(0));
    }

    #[test]
    fn test_match_with_already_partially_filled_order() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(3), // Already partially filled
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(8),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify correct matching considering previous fills
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(5)); // Only 5 more units matched (8-3)
        assert_eq!(sell_order.filled_quantity, dec!(8)); // 3 + 5 = 8
        assert_eq!(sell_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_no_matching_price() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20), // Lower than sell price
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25), // Higher than buy price
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify no matches due to price mismatch
        assert_eq!(matches.len(), 0);
        assert_eq!(sell_order.filled_quantity, dec!(0));
        assert_eq!(sell_order.status, OrderStatus::OPEN);
    }

    #[test]
    fn test_remove_non_existent_order() {
        let mut outcome_book = OutcomeBook::default();

        // Try to remove an order that doesn't exist
        let result = outcome_book.remove_order(get_random_uuid(), OrderSide::BUY, dec!(0.5));
        assert!(!result);

        // Try to remove with wrong side
        let id = get_random_uuid();
        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id,
            price: dec!(0.5),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&order);

        let result = outcome_book.remove_order(id, OrderSide::SELL, dec!(0.5));
        assert!(!result);
        assert_eq!(outcome_book.bids.len(), 1);
    }

    #[test]
    fn test_process_empty_book() {
        let market_id = get_random_uuid();
        let mut outcome_book = OutcomeBook::default();

        let mut order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.5),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        // Process an order when book is empty
        let matches = outcome_book.match_order(&mut order);

        assert_eq!(matches.len(), 0);
        assert_eq!(order.filled_quantity, dec!(0));
        assert_eq!(order.status, OrderStatus::OPEN);
    }

    #[test]
    fn test_market_order_basic_buy() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add a sell order to the book
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create a market buy order
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),    // Market orders have zero price
            quantity: dec!(0), // Will be set by create_market_order
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget for 5 shares: 0.25 * 5 = 1.25
        let budget = dec!(1.25);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(5));
        assert_eq!(matches[0].price, dec!(0.25));
        assert_eq!(market_buy_order.quantity, dec!(5));
        assert_eq!(market_buy_order.filled_quantity, dec!(5));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);

        // Verify the sell order was partially filled
        let price_level = outcome_book.asks.get(&sell_order.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(5)); // 10 - 5 = 5 remaining
    }

    #[test]
    fn test_market_order_basic_sell() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add a buy order to the book
        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.75),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&buy_order);

        // Create a market sell order
        let mut market_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),    // Market orders have zero price
            quantity: dec!(0), // Will be set by create_market_order
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::MARKET,
        };

        // Budget for 5 shares: 0.75 * 5 = 3.75
        let budget = dec!(3.75);
        let matches = outcome_book.create_market_order(&mut market_sell_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(5));
        assert_eq!(matches[0].price, dec!(0.75));
        assert_eq!(market_sell_order.quantity, dec!(5));
        assert_eq!(market_sell_order.filled_quantity, dec!(5));
        assert_eq!(market_sell_order.status, OrderStatus::FILLED);

        // Verify the buy order was partially filled
        let price_level = outcome_book.bids.get(&buy_order.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(5)); // 10 - 5 = 5 remaining
    }

    #[test]
    fn test_market_order_multiple_price_levels_buy() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add multiple sell orders at different prices
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20), // Lowest price, will be matched first
            quantity: dec!(3),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.30), // Middle price
            quantity: dec!(4),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        let sell_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.40), // Highest price, will be matched last
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);
        outcome_book.add_order(&sell_order_3);

        // Create a market buy order
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget enough for some matches: 0.20*3 + 0.30*4 + 0.40*2 = 0.6 + 1.2 + 0.8 = 2.6
        let budget = dec!(2.6);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 3); // 3 price levels matched
        assert_eq!(market_buy_order.quantity, dec!(9)); // 3 + 4 + 2 = 9
        assert_eq!(market_buy_order.filled_quantity, dec!(9));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);

        // Check that the orders were matched in price priority
        assert_eq!(matches[0].price, dec!(0.20)); // Lowest price first
        assert_eq!(matches[0].matched_quantity, dec!(3));

        assert_eq!(matches[1].price, dec!(0.30)); // Middle price second
        assert_eq!(matches[1].matched_quantity, dec!(4));

        assert_eq!(matches[2].price, dec!(0.40)); // Highest price last
        assert_eq!(matches[2].matched_quantity, dec!(2)); // Partial fill

        // Verify the remaining quantity for sell_order_3
        let price_level = outcome_book.asks.get(&sell_order_3.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(3)); // 5 - 2 = 3 remaining
    }

    #[test]
    fn test_market_order_multiple_price_levels_sell() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add multiple buy orders at different prices
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.80), // Highest price, will be matched first
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::LIMIT,
        };

        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.70), // Middle price
            quantity: dec!(4),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::LIMIT,
        };

        let buy_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.60), // Lowest price, will be matched last
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        // Create a market sell order
        let mut market_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::MARKET,
        };

        // Budget enough for some matches: 0.80*3 + 0.70*4 + 0.60*2 = 2.4 + 2.8 + 1.2 = 6.4
        let budget = dec!(6.4);
        let matches = outcome_book.create_market_order(&mut market_sell_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 3); // 3 price levels matched
        assert_eq!(market_sell_order.quantity, dec!(9)); // 3 + 4 + 2 = 9
        assert_eq!(market_sell_order.filled_quantity, dec!(9));
        assert_eq!(market_sell_order.status, OrderStatus::FILLED);

        // Check that the orders were matched in price priority
        assert_eq!(matches[0].price, dec!(0.80)); // Highest price first
        assert_eq!(matches[0].matched_quantity, dec!(3));

        assert_eq!(matches[1].price, dec!(0.70)); // Middle price second
        assert_eq!(matches[1].matched_quantity, dec!(4));

        assert_eq!(matches[2].price, dec!(0.60)); // Lowest price last
        assert_eq!(matches[2].matched_quantity, dec!(2)); // Partial fill

        // Verify the remaining quantity for buy_order_3
        let price_level = outcome_book.bids.get(&buy_order_3.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(3)); // 5 - 2 = 3 remaining
    }

    #[test]
    fn test_market_order_insufficient_budget() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add a sell order to the book
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create a market buy order with insufficient budget
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget for 0 shares
        let budget = dec!(0);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 0);
        assert_eq!(market_buy_order.quantity, dec!(0));
        assert_eq!(market_buy_order.filled_quantity, dec!(0));
        assert_eq!(market_buy_order.status, OrderStatus::CANCELLED);
    }

    #[test]
    fn test_market_order_exact_budget() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add a sell order to the book
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create a market buy order with exact budget
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget for exactly 10 shares: 0.50 * 10 = 5.0
        let budget = dec!(5.0);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(10));
        assert_eq!(matches[0].price, dec!(0.50));
        assert_eq!(market_buy_order.quantity, dec!(10));
        assert_eq!(market_buy_order.filled_quantity, dec!(10));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);

        // Verify the sell order was completely filled
        assert!(!outcome_book.asks.contains_key(&sell_order.price));
    }

    #[test]
    fn test_market_order_same_user_no_match() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid(); // Same user for both orders

        let mut outcome_book = OutcomeBook::default();

        // Add a sell order to the book
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id, // Same user
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create a market buy order from the same user
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id, // Same user
            order_type: OrderType::MARKET,
        };

        let budget = dec!(5.0);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify no matches because same user can't match with themselves
        assert_eq!(matches.len(), 0);
        assert_eq!(market_buy_order.quantity, dec!(0));
        assert_eq!(market_buy_order.status, OrderStatus::CANCELLED);
    }

    #[test]
    fn test_market_order_invalid_order_type() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add a sell order to the book
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create a limit order instead of market order
        let mut limit_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.60),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::LIMIT, // Wrong order type
        };

        let budget = dec!(5.0);
        let matches = outcome_book.create_market_order(&mut limit_buy_order, budget);

        // Verify no matches because wrong order type
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_market_order_mixed_price_levels() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id_1 = get_random_uuid();
        let seller_id_2 = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add multiple sell orders at different prices from different users
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(4),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id_1,
            order_type: OrderType::LIMIT,
        };

        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25), // Same price level
            quantity: dec!(3),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at()
                .checked_add_signed(chrono::Duration::seconds(1))
                .unwrap(), // Later timestamp
            user_id: seller_id_2,
            order_type: OrderType::LIMIT,
        };

        let sell_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.35),
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id_1,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);
        outcome_book.add_order(&sell_order_3);

        // Create a market buy order
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget for all shares at 0.25 and 2 shares at 0.35: (0.25 * 7) + (0.35 * 2) = 1.75 + 0.70 = 2.45
        let budget = dec!(2.45);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 3); // 3 order matches (2 at same price level, 1 at higher price)
        assert_eq!(market_buy_order.quantity, dec!(9)); // 4 + 3 + 2 = 9
        assert_eq!(market_buy_order.filled_quantity, dec!(9));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);

        // Check that the orders were matched in price+time priority
        assert_eq!(matches[0].opposite_order_id, sell_order_1.id); // First order at lowest price
        assert_eq!(matches[0].matched_quantity, dec!(4));

        assert_eq!(matches[1].opposite_order_id, sell_order_2.id); // Second order at lowest price
        assert_eq!(matches[1].matched_quantity, dec!(3));

        assert_eq!(matches[2].opposite_order_id, sell_order_3.id); // Order at higher price
        assert_eq!(matches[2].matched_quantity, dec!(2)); // Partial fill

        // Verify the remaining quantity for sell_order_3
        let price_level = outcome_book.asks.get(&sell_order_3.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(3)); // 5 - 2 = 3 remaining
    }

    #[test]
    fn test_market_order_with_fractional_quantities() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add a sell order to the book with fractional quantity
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10.5), // Fractional quantity
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create a market buy order
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget for exactly 5.25 shares: 0.50 * 5.25 = 2.625
        let budget = dec!(2.625);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(5.25));
        assert_eq!(matches[0].price, dec!(0.50));
        assert_eq!(market_buy_order.quantity, dec!(5.25));
        assert_eq!(market_buy_order.filled_quantity, dec!(5.25));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);

        // Verify the sell order was partially filled
        let price_level = outcome_book.asks.get(&sell_order.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(5.25)); // 10.5 - 5.25 = 5.25 remaining
    }

    #[test]
    fn test_market_book_create_market_order() {
        let market_id = get_random_uuid();
        let buyer_id = get_random_uuid();
        let seller_id = get_random_uuid();

        let mut market_book = MarketBook::new(dec!(100));

        // Add a sell order to the yes book
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.30),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: seller_id,
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&sell_order);

        // Create a market buy order for YES outcome
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: buyer_id,
            order_type: OrderType::MARKET,
        };

        // Budget for 5 shares: 0.30 * 5 = 1.5
        let budget = dec!(1.5);
        let matches = market_book.create_market_order(&mut market_buy_order, budget);

        // Verify the results
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(5));
        assert_eq!(matches[0].price, dec!(0.30));
        assert_eq!(market_buy_order.quantity, dec!(5));
        assert_eq!(market_buy_order.filled_quantity, dec!(5));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);

        // Verify executed_yes_buy_volume was updated
        assert_eq!(market_book.executed_yes_buy_volume, dec!(1.5)); // 0.30 * 5 = 1.5

        // Verify market price was updated
        assert_ne!(market_book.current_yes_price, dec!(0.5)); // Price should have changed
        assert_ne!(market_book.current_no_price, dec!(0.5));
    }

    #[test]
    fn test_get_order_book_functionality() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();

        let mut market_book = MarketBook::new(dec!(100));

        // Add orders to YES book
        let buy_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.40),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id,
            order_type: OrderType::LIMIT,
        };

        let sell_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.60),
            quantity: dec!(3),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id,
            order_type: OrderType::LIMIT,
        };

        // Add orders to NO book
        let buy_order_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::NO,
            price: dec!(0.30),
            quantity: dec!(7),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id,
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&buy_order_yes);
        market_book.add_order(&sell_order_yes);
        market_book.add_order(&buy_order_no);

        // Get the YES order book
        let yes_book = market_book.get_order_book(Outcome::YES).unwrap();

        // Verify YES book structure
        assert_eq!(yes_book.bids.len(), 1);
        assert_eq!(yes_book.asks.len(), 1);

        let yes_bid_level = yes_book.bids.get(&buy_order_yes.price).unwrap();
        assert_eq!(yes_bid_level.total_quantity, dec!(5));

        let yes_ask_level = yes_book.asks.get(&sell_order_yes.price).unwrap();
        assert_eq!(yes_ask_level.total_quantity, dec!(3));

        // Get the NO order book
        let no_book = market_book.get_order_book(Outcome::NO).unwrap();

        // Verify NO book structure
        assert_eq!(no_book.bids.len(), 1);
        assert_eq!(no_book.asks.len(), 0);

        let no_bid_level = no_book.bids.get(&buy_order_no.price).unwrap();
        assert_eq!(no_bid_level.total_quantity, dec!(7));

        // Try to get an invalid order book
        let invalid_book = market_book.get_order_book(Outcome::UNSPECIFIED);
        assert!(invalid_book.is_none());
    }

    #[test]
    fn test_update_order() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();

        let mut market_book = MarketBook::new(dec!(100));

        // Create an initial order
        let mut order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.40),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id,
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&order);

        // Update the order to new price and quantity
        let new_price = dec!(0.45);
        let new_quantity = dec!(7);

        let result = market_book.update_order(&mut order, new_quantity, new_price);

        // Verify the update was successful
        assert!(result);
        assert_eq!(order.price, new_price);
        assert_eq!(order.quantity, new_quantity);
        assert_eq!(order.status, OrderStatus::OPEN);

        // Verify the order book was updated correctly
        let yes_book = market_book.get_order_book(Outcome::YES).unwrap();

        // Old price level should be gone
        assert!(!yes_book.bids.contains_key(&dec!(0.40)));

        // New price level should exist
        assert!(yes_book.bids.contains_key(&new_price));
        let price_level = yes_book.bids.get(&new_price).unwrap();
        assert_eq!(price_level.total_quantity, new_quantity);
    }
}









