use std::{collections::HashMap, sync::{Mutex, mpsc::{self, Sender}}, thread::spawn};
use actix_web::{App, HttpServer, web::{self}};
use futures::channel::oneshot;
use uuid::Uuid;
use crate::{BalanceMessage::{GetBalance, Onramp}, StockMessage::Deposit, routes::user::{balance, deposit, onramp, sign_in, sign_up}, types::user::User};

pub mod types;
pub mod routes;
pub mod middleware;

enum BalanceMessage {
    Onramp(Uuid, u32),
    GetBalance(Uuid, oneshot::Sender<u32>)
}

enum StockMessage {
    Deposit(Uuid, String, u32),
    GetBalances(Uuid, oneshot::Sender<HashMap<String, u32>>)
}

struct AppState {
    users: Mutex<Vec<User>>,
    stock_balances_tx: Sender<StockMessage>,
    usd_balance_tx: Sender<BalanceMessage>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let (usd_tx, usd_rx) = mpsc::channel();
    let (stock_tx, stock_rx) = mpsc::channel();
    
    let app_data = web::Data::new(AppState {
        users: Mutex::new(vec![]),
        stock_balances_tx: stock_tx,
        usd_balance_tx: usd_tx
    });
    
    spawn(move || {
        let mut balances: HashMap<Uuid, u32> = HashMap::new();

        loop { 
            let message = usd_rx.recv().unwrap();
            match message {
                Onramp(user_id, amount) => {
                    let exisiting_balance = balances.get(&user_id).unwrap_or(&0);
                    balances.insert(user_id, exisiting_balance.clone() + amount);
                },
                GetBalance(user_id , tx) => {
                    let new_balance = balances.get(&user_id).unwrap_or(&0);
                    tx.send(*new_balance);
                }
            }
        }
    });

    spawn(move || {
        let mut stock_balances: HashMap<Uuid, HashMap<String, u32>> = HashMap::new();

        loop { 
            let message = stock_rx.recv().unwrap();
            match message {
                StockMessage::Deposit(user_id, symbol, amount) => {
                    let user_balances = stock_balances.entry(user_id).or_insert_with(HashMap::new);
                    let exisiting_balance = user_balances.get(&symbol).unwrap_or(&0).clone();
                    user_balances.insert(symbol, amount + exisiting_balance);
              },
                StockMessage::GetBalances(user_id , tx) => {
                    let empty_stocks = HashMap::new();
                    let all_stocks = stock_balances.get(&user_id).unwrap_or(&empty_stocks);
                    tx.send(all_stocks.clone());
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
