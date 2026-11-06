use std::collections::{BTreeMap, HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum OrderType {
    Bid,
    Ask,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    pub taker: Uuid,
    pub maker: Uuid,
    pub price: u32,
    pub qty: u32,
}

pub struct PriceLevel {
    pub total_qty: u32,
    pub orders: VecDeque<Order>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Order {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub price: u32,
    pub qty: u32,
    pub filled_qty: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOrderReponse {
    pub order_id: Uuid,
    pub matches: Vec<Match>,
    pub filled_qty: u32,
    pub remaining_qty: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CancelledOrder {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub side: OrderType,
    pub price: u32,
    pub remaining_qty: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DepthLevel {
    pub price: u32,
    pub qty: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Depth {
    pub bids: Vec<DepthLevel>,
    pub asks: Vec<DepthLevel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenOrder {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub side: OrderType,
    pub price: u32,
    pub qty: u32,
    pub filled_qty: u32,
}

#[derive(Copy, Clone)]
struct OrderLoc {
    side: OrderType,
    price: u32,
}

pub struct Orderbook {
    pub symbol: String,
    pub asks: BTreeMap<u32, PriceLevel>,
    pub bids: BTreeMap<u32, PriceLevel>,
    index: HashMap<Uuid, OrderLoc>,
}

impl Orderbook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            index: HashMap::new(),
        }
    }

    pub fn create_order(&mut self, order: Order, order_type: OrderType) -> CreateOrderReponse {
        let mut remaining = order.qty;
        let mut matches: Vec<Match> = Vec::new();

        match order_type {
            OrderType::Bid => {
                while remaining > 0 {
                    let best_ask = match self.asks.keys().next().cloned() {
                        Some(p) => p,
                        None => break,
                    };

                    if best_ask > order.price {
                        break;
                    }

                    let (remove_level, filled_ids, stop_for_self) = {
                        let level = self.asks.get_mut(&best_ask).unwrap();
                        let mut filled_ids = Vec::new();
                        let mut stop_for_self = false;

                        while remaining > 0 && !level.orders.is_empty() {
                            if level.orders.front().unwrap().user_id == order.user_id {
                                stop_for_self = true;
                                break;
                            }

                            let ask_order = level.orders.front_mut().unwrap();
                            let resting_qty = ask_order.qty - ask_order.filled_qty;
                            let match_qty = remaining.min(resting_qty);

                            level.total_qty -= match_qty;
                            remaining -= match_qty;
                            ask_order.filled_qty += match_qty;

                            matches.push(Match {
                                taker: order.user_id,
                                maker: ask_order.user_id,
                                price: best_ask,
                                qty: match_qty,
                            });

                            if ask_order.filled_qty == ask_order.qty {
                                let done = level.orders.pop_front().unwrap();
                                filled_ids.push(done.order_id);
                            }
                        }
                        (level.total_qty == 0, filled_ids, stop_for_self)
                    };

                    for id in filled_ids {
                        self.index.remove(&id);
                    }
                    if remove_level {
                        self.asks.remove(&best_ask);
                    }
                    if stop_for_self {
                        break;
                    }
                }
            }
            OrderType::Ask => {
                while remaining > 0 {
                    let best_bid = match self.bids.keys().next_back().cloned() {
                        Some(p) => p,
                        None => break,
                    };

                    if best_bid < order.price {
                        break;
                    }

                    let (remove_level, filled_ids, stop_for_self) = {
                        let level = self.bids.get_mut(&best_bid).unwrap();
                        let mut filled_ids = Vec::new();
                        let mut stop_for_self = false;

                        while remaining > 0 && !level.orders.is_empty() {
                            if level.orders.front().unwrap().user_id == order.user_id {
                                stop_for_self = true;
                                break;
                            }

                            let bid_order = level.orders.front_mut().unwrap();
                            let resting_qty = bid_order.qty - bid_order.filled_qty;
                            let match_qty = remaining.min(resting_qty);

                            level.total_qty -= match_qty;
                            remaining -= match_qty;
                            bid_order.filled_qty += match_qty;

                            matches.push(Match {
                                taker: order.user_id,
                                maker: bid_order.user_id,
                                price: best_bid,
                                qty: match_qty,
                            });

                            if bid_order.filled_qty == bid_order.qty {
                                let done = level.orders.pop_front().unwrap();
                                filled_ids.push(done.order_id);
                            }
                        }
                        (level.total_qty == 0, filled_ids, stop_for_self)
                    };

                    for id in filled_ids {
                        self.index.remove(&id);
                    }
                    if remove_level {
                        self.bids.remove(&best_bid);
                    }
                    if stop_for_self {
                        break;
                    }
                }
            }
        }

        if remaining > 0 {
            let open_order = Order {
                order_id: order.order_id,
                user_id: order.user_id,
                price: order.price,
                qty: remaining,
                filled_qty: 0,
            };
            self.index.insert(
                open_order.order_id,
                OrderLoc {
                    side: order_type,
                    price: open_order.price,
                },
            );

            match order_type {
                OrderType::Ask => {
                    let level = self.asks.entry(order.price).or_insert(PriceLevel {
                        total_qty: 0,
                        orders: VecDeque::new(),
                    });
                    level.total_qty += remaining;
                    level.orders.push_back(open_order);
                }
                OrderType::Bid => {
                    let level = self.bids.entry(order.price).or_insert(PriceLevel {
                        total_qty: 0,
                        orders: VecDeque::new(),
                    });
                    level.total_qty += remaining;
                    level.orders.push_back(open_order);
                }
            }
        }

        CreateOrderReponse {
            order_id: order.order_id,
            matches,
            filled_qty: order.qty - remaining,
            remaining_qty: remaining,
        }
    }

    pub fn cancel(&mut self, user_id: Uuid, order_id: Uuid) -> Option<CancelledOrder> {
        let loc = *self.index.get(&order_id)?;
        let cancelled = {
            let levels = match loc.side {
                OrderType::Bid => &mut self.bids,
                OrderType::Ask => &mut self.asks,
            };
            let level = levels.get_mut(&loc.price)?;
            let pos = level.orders.iter().position(|o| o.order_id == order_id)?;
            if level.orders[pos].user_id != user_id {
                return None;
            }
            let order = level.orders.remove(pos).unwrap();
            let remaining_qty = order.qty - order.filled_qty;
            level.total_qty -= remaining_qty;
            if level.total_qty == 0 {
                levels.remove(&loc.price);
            }
            CancelledOrder {
                order_id,
                user_id: order.user_id,
                side: loc.side,
                price: order.price,
                remaining_qty,
            }
        };
        self.index.remove(&order_id);
        Some(cancelled)
    }

    pub fn depth(&self) -> Depth {
        Depth {
            bids: self
                .bids
                .iter()
                .rev()
                .map(|(price, level)| DepthLevel {
                    price: *price,
                    qty: level.total_qty,
                })
                .collect(),
            asks: self
                .asks
                .iter()
                .map(|(price, level)| DepthLevel {
                    price: *price,
                    qty: level.total_qty,
                })
                .collect(),
        }
    }

    pub fn open_orders(&self, user_id: Uuid) -> Vec<OpenOrder> {
        let mut orders = Vec::new();
        for level in self.bids.values() {
            for order in &level.orders {
                if order.user_id == user_id {
                    orders.push(open_order(order, OrderType::Bid));
                }
            }
        }
        for level in self.asks.values() {
            for order in &level.orders {
                if order.user_id == user_id {
                    orders.push(open_order(order, OrderType::Ask));
                }
            }
        }
        orders
    }
}

fn open_order(order: &Order, side: OrderType) -> OpenOrder {
    OpenOrder {
        order_id: order.order_id,
        user_id: order.user_id,
        side,
        price: order.price,
        qty: order.qty,
        filled_qty: order.filled_qty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_for(user_id: Uuid, price: u32, qty: u32) -> Order {
        Order {
            order_id: Uuid::new_v4(),
            user_id,
            price,
            qty,
            filled_qty: 0,
        }
    }

    fn order(price: u32, qty: u32) -> Order {
        order_for(Uuid::new_v4(), price, qty)
    }

    #[test]
    fn bid_does_not_cross_a_higher_ask() {
        let mut book = Orderbook::new("SOL".into());
        book.create_order(order(100, 5), OrderType::Ask);
        let res = book.create_order(order(90, 5), OrderType::Bid);
        assert!(res.matches.is_empty());
        assert_eq!(res.filled_qty, 0);
        assert_eq!(res.remaining_qty, 5);
        assert_eq!(book.bids.get(&90).unwrap().total_qty, 5);
    }

    #[test]
    fn bid_fills_at_the_ask_price() {
        let mut book = Orderbook::new("SOL".into());
        book.create_order(order(100, 4), OrderType::Ask);
        let res = book.create_order(order(110, 10), OrderType::Bid);
        assert_eq!(res.filled_qty, 4);
        assert_eq!(res.remaining_qty, 6);
        assert_eq!(res.matches[0].price, 100);
        assert_eq!(book.bids.get(&110).unwrap().total_qty, 6);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn cancel_removes_the_resting_order() {
        let mut book = Orderbook::new("SOL".into());
        let user_id = Uuid::new_v4();
        let created = book.create_order(order_for(user_id, 100, 5), OrderType::Ask);

        let cancelled = book.cancel(user_id, created.order_id).unwrap();
        assert_eq!(cancelled.remaining_qty, 5);
        assert_eq!(cancelled.price, 100);
        assert!(book.asks.is_empty());
        assert!(book.cancel(user_id, created.order_id).is_none());
    }

    #[test]
    fn cancel_leaves_another_users_order() {
        let mut book = Orderbook::new("SOL".into());
        let owner = Uuid::new_v4();
        let created = book.create_order(order_for(owner, 100, 5), OrderType::Bid);
        assert!(book.cancel(Uuid::new_v4(), created.order_id).is_none());
        assert_eq!(book.bids.get(&100).unwrap().total_qty, 5);
    }

    #[test]
    fn partial_fill_can_be_cancelled_for_the_remainder() {
        let mut book = Orderbook::new("SOL".into());
        let seller = Uuid::new_v4();
        let buyer = Uuid::new_v4();
        let created = book.create_order(order_for(seller, 100, 10), OrderType::Ask);
        book.create_order(order_for(buyer, 100, 4), OrderType::Bid);

        let cancelled = book.cancel(seller, created.order_id).unwrap();
        assert_eq!(cancelled.remaining_qty, 6);
        assert!(book.asks.is_empty());
    }

    #[test]
    fn self_trade_does_not_match() {
        let mut book = Orderbook::new("SOL".into());
        let user_id = Uuid::new_v4();
        book.create_order(order_for(user_id, 100, 5), OrderType::Ask);
        let res = book.create_order(order_for(user_id, 100, 5), OrderType::Bid);
        assert!(res.matches.is_empty());
        assert_eq!(book.asks.get(&100).unwrap().total_qty, 5);
        assert_eq!(book.bids.get(&100).unwrap().total_qty, 5);
    }

    #[test]
    fn depth_lists_best_prices_first() {
        let mut book = Orderbook::new("SOL".into());
        book.create_order(order(100, 2), OrderType::Bid);
        book.create_order(order(90, 3), OrderType::Bid);
        book.create_order(order(110, 4), OrderType::Ask);
        book.create_order(order(120, 1), OrderType::Ask);

        let depth = book.depth();
        assert_eq!(depth.bids[0].price, 100);
        assert_eq!(depth.bids[1].price, 90);
        assert_eq!(depth.asks[0].price, 110);
        assert_eq!(depth.asks[0].qty, 4);
    }
}
