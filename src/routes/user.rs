use actix_web::{HttpResponse, Responder, post, web::{self, Json}};

use crate::{AppState, types::auth::{SignupInputs, SignupResponse, User}};

#[post("/signup")]
pub async fn sign_up(body: Json<SignupInputs>, app_data: web::Data<AppState>) -> impl Responder {
    let mut data = app_data.users.lock().unwrap();
    let found_user = data.iter().find(|u| u.email == body.email);
    if found_user.is_none() {
        data.push(User {
            email: body.email.clone(),
            // should ideally be hased
            password: body.password.clone()
        });

        HttpResponse::Ok().json(SignupResponse {
            message: String::from("User created!")
        })
    } else {
        HttpResponse::Conflict().json(SignupResponse {
            message: String::from("User Already Exists")
        })
    }
    
}

#[post("/signin")]
pub async fn sign_in(body: Json<SignupInputs>, app_data: web::Data<AppState>) -> impl Responder {
    let mut data = app_data.users.lock().unwrap();
    let found_user = data.iter().find(|u| u.email == body.email);

    match found_user {
        Some(user) => {
            if user.password == body.password {
                return HttpResponse::Ok().json(SignupResponse {
                    message: String::from("Correct.")
                });
            } else {
                return HttpResponse::NotAcceptable().json(SignupResponse {
                    message: String::from("incorrect creds")
                });
            }
        },
        None => {
            return HttpResponse::NotFound().json(SignupResponse {
                message: String::from("User not found")
            });
        } 
    }    
}