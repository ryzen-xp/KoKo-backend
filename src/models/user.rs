use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
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