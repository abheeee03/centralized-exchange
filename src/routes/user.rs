use std::{
    collections::HashMap, sync::mpsc, time::{SystemTime, UNIX_EPOCH},
};
use crate::{
    AppState, BalanceMessage, middleware::auth::AuthUser, types::user::{
        AssetDepositInput, BalanceResponse, Cliams, DepositResponse, OnRampInput, SignInResponse, SignupInputs, SignupResponse, User,
    },
};
use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Json},
};
use futures::channel::oneshot;
use jsonwebtoken::{EncodingKey, Header, encode};
use uuid::Uuid;

#[post("/signup")]
pub async fn sign_up(body: Json<SignupInputs>, app_data: web::Data<AppState>) -> impl Responder {
    let mut users_data = app_data.users.lock().unwrap();
    let found_user = users_data.iter().find(|u| u.email == body.email);
    if found_user.is_none() {
        let new_id = Uuid::new_v4();
        let mut stock_balances = app_data.stock_balances.lock().unwrap();
        users_data.push(User {
            id: new_id,
            email: body.email.clone(),
            // should ideally be hased
            password: body.password.clone(),
        });
        app_data.usd_balance_tx.send(BalanceMessage::Onramp(new_id, 0));
        stock_balances.insert(new_id, HashMap::new());

        HttpResponse::Ok().json(SignupResponse {
            message: String::from("User created!"),
        })
    } else {
        HttpResponse::Conflict().json(SignupResponse {
            message: String::from("User Already Exists"),
        })
    }
}

#[post("/signin")]
pub async fn sign_in(body: Json<SignupInputs>, app_data: web::Data<AppState>) -> impl Responder {
    let data = app_data.users.lock().unwrap();

    let Some(user) = data.iter().find(|u| u.email == body.email) else {
        return HttpResponse::NotFound().json(SignInResponse {
            token: Option::None,
            message: String::from("User not found"),
        });
    };

    if user.password == body.password {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60 * 60 * 24;
        let claims = Cliams { sub: user.id, exp };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("secret".as_ref()),
        )
        .unwrap();
        return HttpResponse::Ok().json(SignInResponse {
            message: String::from("Ok"),
            token: Some(token),
        });
    } else {
        return HttpResponse::Unauthorized().json(SignInResponse {
            message: String::from("Invalid Creds"),
            token: Option::None,
        });
    }
}

#[get("/balance")]
pub async fn balance(app_data: web::Data<AppState>, user: AuthUser) -> impl Responder {
    let user_id = user.0;
    
    let (tx, rx) = oneshot::channel::<u32>();

    app_data.usd_balance_tx.send(BalanceMessage::GetBalance(user_id, tx));
    
    let usd_balance =  rx.await.unwrap();
    
    let stock_balance = app_data
        .stock_balances
        .lock()
        .unwrap()
        .get(&user_id)
        .unwrap_or(&HashMap::new())
        .clone();
    HttpResponse::Ok().json(BalanceResponse {
        usd_balance,
        stock_balance,
    })
}

#[post("/onramp")]
pub async fn onramp(
    body: Json<OnRampInput>,
    user: AuthUser,
    app_data: web::Data<AppState>,
) -> impl Responder {
    app_data.usd_balance_tx.send(BalanceMessage::Onramp(user.0, body.amount));
    HttpResponse::Ok()
}

#[post("/deposit/{symbol}")]
pub async fn deposit(app_state: web::Data<AppState>, body: Json<AssetDepositInput>, user: AuthUser, symbol: web::Path<String>) -> impl Responder {
    let user_id = user.0;
    let mut stock_balances = app_state.stock_balances.lock().unwrap();
    let sym = symbol.into_inner();

    let user_balances = stock_balances.entry(user_id).or_insert_with(HashMap::new);
    let exisiting_balances = user_balances.get(&sym).unwrap_or(&0).clone();

    user_balances.insert(sym, exisiting_balances + body.qty);
    HttpResponse::Ok().json(DepositResponse {
        message: String::from("Deposit Successful")
    })
}