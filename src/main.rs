use crate::{
    routes::user::{balance, deposit, onramp, sign_in, sign_up},
    types::user::User,
};
use actix_web::{
    App, HttpServer,
    web::{self},
};
use futures::channel::oneshot;
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        mpsc::{self, Sender},
    },
    thread::spawn,
};
use uuid::Uuid;

pub mod middleware;
pub mod routes;
pub mod types;

enum AppMessage {
    OnrampUsd(Uuid, u32),
    GetUsdBalance(Uuid, oneshot::Sender<u32>),
    DepositStock(Uuid, String, u32),
    GetStockBalances(Uuid, oneshot::Sender<HashMap<String, u32>>),
}

struct AppState {
    users: Mutex<Vec<User>>,
    balances_tx: Sender<AppMessage>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel();

    let app_data = web::Data::new(AppState {
        users: Mutex::new(vec![]),
        balances_tx: tx,
    });

    spawn(move || {
        let mut stock_balances: HashMap<Uuid, HashMap<String, u32>> = HashMap::new();
        let mut usd_balances: HashMap<Uuid, u32> = HashMap::new();
        
        loop {
            let message = rx.recv().unwrap();
            match message {
                AppMessage::OnrampUsd(user_id, amount) => {
                    let exisiting_balance = usd_balances.get(&user_id).unwrap_or(&0);
                    usd_balances.insert(user_id, exisiting_balance.clone() + amount);
                }
                AppMessage::GetUsdBalance(user_id, tx) => {
                    let new_balance = usd_balances.get(&user_id).unwrap_or(&0);
                    tx.send(*new_balance);
                }
                AppMessage::DepositStock(user_id, symbol, amount) => {
                    let user_balances = stock_balances.entry(user_id).or_insert_with(HashMap::new);
                    let exisiting_balance = user_balances.get(&symbol).unwrap_or(&0).clone();
                    user_balances.insert(symbol, amount + exisiting_balance);
                }
                AppMessage::GetStockBalances(user_id, tx) => {
                    let empty_stocks = HashMap::new();
                    let all_stocks = stock_balances.get(&user_id).unwrap_or(&empty_stocks);
                    let _  = tx.send(all_stocks.clone());
                }
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(sign_up)
            .service(sign_in)
            .service(balance)
            .service(onramp)
            .service(deposit)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
