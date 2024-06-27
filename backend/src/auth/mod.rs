use serde::Deserialize;

pub(crate) mod token_entity;
pub(crate) mod auth_handler;

#[derive(Deserialize)]
pub struct RegisterUser {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub(crate) id: String,
    pub(crate) password: String,
}