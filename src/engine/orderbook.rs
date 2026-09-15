use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum OrderType {
    Bid,
    Ask,
}

#[derive(Serialize, Deserialize)]
pub struct Match {
    taker: Uuid,
    maker: Uuid,
    price: u32,
    qty: u32
}

pub struct Ask {
    total_qty: u32,
    orders: Vec<Order>
}
pub struct Bid {
    total_qty: u32,
    orders: Vec<Order>
}

pub struct Order {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub qty: u32,
    pub filled_qty: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CreateOrderReponse {
    matches: Vec<Match>,
    filled_qty: u32
}

pub struct Orderbook {
    pub symbol: String,
    pub asks: BTreeMap<u32, Ask>,
    pub bids: BTreeMap<u32, Bid>,
}

impl Orderbook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
        }
    }

    pub fn create_order(&mut self, order: Order, price: u32, order_type: OrderType) -> CreateOrderReponse {
        let mut remaining = order.qty;
        let mut matches: Vec<Match> = Vec::new();

        match order_type {
            OrderType::Bid => {
                while remaining > 0 {
                    let best_ask = match self.asks.keys().next().cloned() {
                        Some(p) => p,
                        None => break
                    };
                    
                    let remove_level = {
                        let level = self.asks.get_mut(&best_ask).unwrap();

                        while remaining > 0 && !level.orders.is_empty() {
                            let ask_order = &mut level.orders[0];
                            let resting_qty = ask_order.qty - ask_order.filled_qty;
                            let match_qty = remaining.min(resting_qty);

                            level.total_qty -= match_qty;
                            remaining -= match_qty;
                            ask_order.filled_qty += match_qty;

                            matches.push(Match {
                                taker: order.user_id,
                                maker: ask_order.user_id,
                                price: best_ask,
                                qty: match_qty
                            });

                            if ask_order.filled_qty == ask_order.qty {
                                level.orders.remove(0);
                            }
                        }
                        level.total_qty == 0
                    };
                    
                    if remove_level {
                        self.asks.remove(&best_ask);
                    }
                    
                }
            }
            OrderType::Ask => {
                while remaining > 0 {
                    let best_bid = match self.bids.keys().next_back().cloned() {
                        Some(p) => p,
                        None => break
                    };
                    
                    let remove_level = {
                        let level = self.bids.get_mut(&best_bid).unwrap();

                        while remaining > 0 && !level.orders.is_empty() {
                            let ask_order = &mut level.orders[0];
                            let resting_qty = ask_order.qty - ask_order.filled_qty;
                            let match_qty = remaining.min(resting_qty);

                            level.total_qty -= match_qty;
                            remaining -= match_qty;
                            ask_order.filled_qty += match_qty;

                            matches.push(Match {
                                taker: order.user_id,
                                maker: ask_order.user_id,
                                price: best_bid,
                                qty: match_qty
                            });

                            if ask_order.filled_qty == ask_order.qty {
                                level.orders.remove(0);
                            }
                        }
                        level.total_qty == 0
                    };
                    
                    if remove_level {
                        self.bids.remove(&best_bid);
                    }   
                }
            }
        }

        if remaining > 0 {
            let open_order = Order {
                order_id: order.order_id,
                user_id: order.user_id,
                qty: remaining,
                filled_qty: 0
            };

            match order_type {
                OrderType::Ask => {
                    let level = self.asks.entry(price).or_insert(Ask {
                        total_qty: 0,
                        orders: Vec::new()
                    });

                    level.total_qty += remaining;
                    level.orders.push(open_order);
                },
                OrderType::Bid => {
                    let level = self.bids.entry(price).or_insert(Bid {
                        total_qty: 0,
                        orders: Vec::new()
                    });

                    level.total_qty += remaining;
                    level.orders.push(open_order);
                },
            }
        }
        CreateOrderReponse {
            matches,
            filled_qty: remaining
        }
    }    
}

