pub mod entities {
    pub mod image_entity;
    pub mod session_entity;
    pub mod user_entity;
}
pub mod errors;
pub mod ports {
    pub mod auth_provider;
    pub mod image_repository;
    pub mod session_repository;
    pub mod user_repository;
}
pub mod services {
    pub mod image_service;
    pub mod session_service;
    pub mod user_service;
}
pub mod types;
pub mod use_cases {
    pub mod authentication;
}
