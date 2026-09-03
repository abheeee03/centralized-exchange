use std::{collections::HashMap, sync::{Mutex, mpsc::{self, Sender}}, thread::spawn};
use actix_web::{App, HttpServer, web::{self}};
use futures::channel::oneshot;
use uuid::Uuid;
use crate::{BalanceMessage::{GetBalance, Onramp}, routes::user::{balance, deposit, onramp, sign_in, sign_up}, types::user::User};

pub mod types;
pub mod routes;
pub mod middleware;

enum BalanceMessage {
    Onramp(Uuid, u32),
    GetBalance(Uuid, oneshot::Sender<u32>)
}

struct AppState {
    users: Mutex<Vec<User>>,
    stock_balances: Mutex<HashMap<Uuid, HashMap<String, u32>>>,
    usd_balance_tx: Sender<BalanceMessage>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let (tx, rx) = mpsc::channel();
    
    let app_data = web::Data::new(AppState {
        users: Mutex::new(vec![]),
        stock_balances: Mutex::new(HashMap::new()),
        usd_balance_tx: tx
    });
    
    spawn(move || {
        let mut balances: HashMap<Uuid, u32> = HashMap::new();

        loop { 
            let message = rx.recv().unwrap();
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
