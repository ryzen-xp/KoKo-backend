use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub handle: String,
    pub email: String,
    pub password_hash: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub is_verified: bool,
    pub is_private: bool,
    pub follower_count: i32,
    pub following_count: i32,
    pub tweet_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub handle: String,
    pub email: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub is_verified: bool,
    pub is_private: bool,
    pub follower_count: i32,
    pub following_count: i32,
    pub tweet_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            handle: user.handle,
            email: user.email,
            bio: user.bio,
            avatar_url: user.avatar_url,
            banner_url: user.banner_url,
            is_verified: user.is_verified,
            is_private: user.is_private,
            follower_count: user.follower_count,
            following_count: user.following_count,
            tweet_count: user.tweet_count,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}


//  DTO's of Users 

#[derive(Debug , Deserialize)]
pub struct  CreateUserDto {
    pub username : String , 
    pub  handle : String , 
    pub email : String , 
    pub password : String 
}


pub struct UpdateUserDto {
    pub username :Option<String> , 
    pub bio : Option<String> , 
    pub avatar_url : Option<String> ,
    pub banner_utl : Option<String> , 
    pub is_private : Option<String>
}
