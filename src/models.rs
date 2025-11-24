use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// 用户表结构
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 创建用户表的SQL
pub const CREATE_USER_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
"#;

// 插入用户的SQL
pub const INSERT_USER_SQL: &str = r#"
INSERT INTO users (username, email) VALUES (?, ?)
"#;

// 查询所有用户的SQL
pub const SELECT_ALL_USERS_SQL: &str = r#"
SELECT id, username, email, created_at, updated_at FROM users
"#;

// 根据ID查询用户的SQL
pub const SELECT_USER_BY_ID_SQL: &str = r#"
SELECT id, username, email, created_at, updated_at FROM users WHERE id = ?
"#;

// 更新用户的SQL
pub const UPDATE_USER_SQL: &str = r#"
UPDATE users SET email = ? WHERE id = ?
"#;

// 删除用户的SQL
pub const DELETE_USER_SQL: &str = r#"
DELETE FROM users WHERE id = ?
"#;

// Profile 表结构
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: u64,
    pub user_id: u64,
    pub full_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 创建 profile 表的SQL
pub const CREATE_PROFILE_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS profiles (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT UNSIGNED NOT NULL UNIQUE,
    full_name VARCHAR(100) NOT NULL,
    bio TEXT,
    avatar_url VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
"#;

// 插入 profile 的SQL
pub const INSERT_PROFILE_SQL: &str = r#"
INSERT INTO profiles (user_id, full_name, bio, avatar_url) VALUES (?, ?, ?, ?)
"#;

// 查询所有 profiles 的SQL
pub const SELECT_ALL_PROFILES_SQL: &str = r#"
SELECT id, user_id, full_name, bio, avatar_url, created_at, updated_at FROM profiles
"#;

// 根据 user_id 查询 profile 的SQL
pub const SELECT_PROFILE_BY_USER_ID_SQL: &str = r#"
SELECT id, user_id, full_name, bio, avatar_url, created_at, updated_at FROM profiles WHERE user_id = ?
"#;

// 更新 profile 的SQL
pub const UPDATE_PROFILE_SQL: &str = r#"
UPDATE profiles SET full_name = ?, bio = ?, avatar_url = ? WHERE user_id = ?
"#;

// 删除 profile 的SQL
pub const DELETE_PROFILE_SQL: &str = r#"
DELETE FROM profiles WHERE user_id = ?
"#;