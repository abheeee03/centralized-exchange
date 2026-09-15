use actix_web::{
    HttpResponse, Responder, post,
    web::{self, Json},
};
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};

use crate::{AppMessage, AppState, engine::orderbook::{CreateOrderReponse, OrderType}, middleware::auth::AuthUser};

#[derive(Serialize, Deserialize)]
struct CreateOrderBody {
    pub side: OrderType,
    pub price: u32,
    pub qty: u32,
}

#[derive(Serialize, Deserialize)]
struct CreateOrderHttpResponse {
    pub data: CreateOrderReponse
}

#[post("/order")]
pub async fn create_order(body: Json<CreateOrderBody>, app_data: web::Data<AppState>, user: AuthUser) -> impl Responder {
    let user_id = user.0;

    let (tx, rx) = oneshot::channel::<CreateOrderReponse>();
    let _ = app_data
        .tx
        .send(AppMessage::CreateOrder(user_id, body.side, body.price, body.qty, tx));
    let res = rx.await.unwrap();

    HttpResponse::Ok().json(CreateOrderHttpResponse {
        data: res
    })
}
