use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Json},
};
use engine::orderbook::{CancelledOrder, CreateOrderReponse, Depth, OpenOrder, OrderType};
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppMessage, AppState, middleware::auth::AuthUser};

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

#[derive(Deserialize)]
struct CancelOrderBody {
    pub order_id: Uuid,
}

#[derive(Serialize)]
struct CancelOrderHttpResponse {
    pub data: CancelledOrder,
}

#[post("/order/cancel")]
pub async fn cancel_order(
    body: Json<CancelOrderBody>,
    app_data: web::Data<AppState>,
    user: AuthUser,
) -> impl Responder {
    let (tx, rx) = oneshot::channel::<Option<CancelledOrder>>();
    let _ = app_data
        .tx
        .send(AppMessage::CancelOrder(user.0, body.order_id, tx));

    match rx.await.unwrap() {
        Some(order) => HttpResponse::Ok().json(CancelOrderHttpResponse { data: order }),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "message": "Order not found"
        })),
    }
}

#[get("/depth")]
pub async fn depth(app_data: web::Data<AppState>, _user: AuthUser) -> impl Responder {
    let (tx, rx) = oneshot::channel::<Depth>();
    let _ = app_data.tx.send(AppMessage::GetDepth(tx));
    let book = rx.await.unwrap();
    HttpResponse::Ok().json(book)
}

#[get("/orders")]
pub async fn open_orders(app_data: web::Data<AppState>, user: AuthUser) -> impl Responder {
    let (tx, rx) = oneshot::channel::<Vec<OpenOrder>>();
    let _ = app_data
        .tx
        .send(AppMessage::GetOpenOrders(user.0, tx));
    let orders = rx.await.unwrap();
    HttpResponse::Ok().json(orders)
}
