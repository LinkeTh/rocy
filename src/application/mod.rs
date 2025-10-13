pub mod entities {
    pub mod image_entity;
    pub mod receipt_entity;
    pub mod session_entity;
    pub mod user_entity;
}
pub mod errors;

pub mod types;

pub mod ports {
    pub mod auth_provider;
    pub mod image_repository;
    pub mod receipt_repository;
    pub mod session_repository;
    pub mod user_repository;
}
