use crate::application::entities::user_entity::UserEntity;
use crate::application::errors::AppError;
use crate::application::ports::user_repository::UserRepository;
use crate::application::types::{Email, Provider, Sub, UserId};

#[derive(Clone)]
pub struct UserService<D: UserRepository> {
    user_repo: D,
}

impl<D: UserRepository> UserService<D> {
    pub fn new(user_repo: D) -> Self {
        Self { user_repo }
    }

    pub async fn create_or_update_user(&self, provider: &Provider, sub: &Sub, email: &Email) -> Result<UserId, AppError> {
        let user_id = self.user_repo.upsert_user(provider, sub, email).await?;
        Ok(user_id)
    }

    pub async fn find_user(&self, user_id: &UserId) -> Result<Option<UserEntity>, AppError> {
        let user = self.user_repo.find_user_by_id(user_id).await?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::entities::user_entity::UserEntity;
    use crate::application::ports::user_repository::UserRepository;
    use crate::application::types::{Email, Provider, Sub, UserId};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct InMemoryUserRepo {
        by_identity: Arc<Mutex<HashMap<(Provider, Sub), UserId>>>,
        users: Arc<Mutex<HashMap<UserId, UserEntity>>>,
        next_id: Arc<Mutex<i32>>,
    }

    impl InMemoryUserRepo {
        fn new() -> Self {
            Self::default()
        }
        fn next_user_id(&self) -> UserId {
            let mut guard = self.next_id.lock().unwrap();
            let id = *guard + 1;
            *guard = id;
            UserId(id)
        }
    }

    #[async_trait]
    impl UserRepository for InMemoryUserRepo {
        async fn upsert_user(&self, provider: &Provider, sub: &Sub, email: &Email) -> Result<UserId, AppError> {
            let p = provider.clone();
            let s = sub.clone();

            let key = (provider.to_owned(), sub.to_owned());

            // check existing by (provider, sub)
            if let Some(existing_id) = self.by_identity.lock().unwrap().get(&key).cloned() {
                // update email on existing user
                if let Some(user) = self.users.lock().unwrap().get_mut(&existing_id) {
                    user.email = email.to_owned();
                }
                return Ok(existing_id);
            }

            // create new user
            let new_id = self.next_user_id();
            self.by_identity.lock().unwrap().insert(key, new_id);
            self.users.lock().unwrap().insert(
                new_id,
                UserEntity {
                    id: new_id,
                    provider: p,
                    sub: s,
                    email: email.to_owned(),
                    created: Default::default(),
                },
            );
            Ok(new_id)
        }

        async fn find_user_by_id(&self, user_id: &UserId) -> Result<Option<UserEntity>, AppError> {
            Ok(self.users.lock().unwrap().get(&user_id).cloned())
        }
    }

    #[tokio::test]
    async fn update_user_creates_new_user_on_first_seen_identity() {
        let repo = InMemoryUserRepo::new();
        let service = UserService::new(repo.clone());

        let id = service
            .create_or_update_user(&Provider("google".into()), &Sub("sub-123".into()), &Email("u@example.com".into()))
            .await
            .expect("upsert should succeed");

        assert!(id.0 > 0);

        let fetched = service.find_user(&id).await.expect("get_user ok");
        let fetched = fetched.expect("user should exist");
        assert_eq!(fetched.id.0, id.0);
        assert_eq!(fetched.email.0, "u@example.com");
    }

    #[tokio::test]
    async fn update_user_upserts_same_identity_and_updates_email() {
        let repo = InMemoryUserRepo::new();
        let service = UserService::new(repo.clone());

        let id1 = service
            .create_or_update_user(&Provider("google".into()), &Sub("sub-abc".into()), &Email("first@example.com".into()))
            .await
            .unwrap();

        let id2 = service
            .create_or_update_user(&Provider("google".into()), &Sub("sub-abc".into()), &Email("second@example.com".into()))
            .await
            .unwrap();

        assert_eq!(id1.0, id2.0, "the same identity should upsert the same user id");

        let fetched = service.find_user(&id1).await.unwrap().unwrap();
        assert_eq!(fetched.email.0, "second@example.com", "email updated on upsert");
    }

    #[tokio::test]
    async fn get_user_returns_none_for_unknown_id() {
        let repo = InMemoryUserRepo::new();
        let service = UserService::new(repo);

        let unknown = UserId(9999);
        let fetched = service.find_user(&unknown).await.unwrap();
        assert!(fetched.is_none());
    }
}
