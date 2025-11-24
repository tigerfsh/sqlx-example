use anyhow::Result;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::env;
use tracing::{debug, error, info};

use crate::models::{User, CREATE_USER_TABLE_SQL};

// 创建数据库连接池
pub async fn create_pool() -> Result<Pool<MySql>> {
    // 从环境变量获取数据库URL，如果没有设置则使用默认值
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:Fsh_2021@localhost:3306/airflow".to_string());

    info!("连接数据库: {}", database_url);

    // 创建数据库连接池 - 禁用 SSL/TLS
    let pool = match MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            info!("数据库连接成功!");
            pool
        }
        Err(e) => {
            error!("数据库连接失败: {}", e);
            error!("尝试禁用 SSL/TLS 连接...");

            // 尝试禁用 SSL 连接
            let database_url_no_ssl = format!("{}?ssl-mode=disabled", database_url);
            match MySqlPoolOptions::new()
                .max_connections(5)
                .connect(&database_url_no_ssl)
                .await
            {
                Ok(pool) => {
                    info!("数据库连接成功 (禁用SSL)!");
                    pool
                }
                Err(e2) => {
                    error!("禁用SSL后连接仍然失败: {}", e2);
                    error!("请检查: 1. MySQL服务是否运行 2. 数据库是否存在 3. 用户名密码是否正确");
                    return Err(e2.into());
                }
            }
        }
    };

    Ok(pool)
}

// 创建用户表
#[tracing::instrument]
pub async fn create_table(pool: &Pool<MySql>) -> Result<()> {
    info!("开始创建用户表");
    sqlx::query(CREATE_USER_TABLE_SQL).execute(pool).await?;
    info!("用户表创建成功");
    Ok(())
}

// 查询所有用户
#[tracing::instrument]
pub async fn select_all_users(pool: &Pool<MySql>) -> Result<Vec<User>> {
    debug!("开始查询所有用户");
    let users = sqlx::query_as::<_, User>(crate::models::SELECT_ALL_USERS_SQL)
        .fetch_all(pool)
        .await?;
    debug!("查询到 {} 个用户", users.len());
    Ok(users)
}

// 根据ID查询用户
#[tracing::instrument]
pub async fn select_user_by_id(pool: &Pool<MySql>, id: u64) -> Result<Option<User>> {
    debug!("根据ID查询用户 - ID: {}", id);
    let user = sqlx::query_as::<_, User>(crate::models::SELECT_USER_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await?;

    if user.is_some() {
        debug!("找到用户 - ID: {}", id);
    } else {
        debug!("未找到用户 - ID: {}", id);
    }
    Ok(user)
}

// 查找最早的用户
#[tracing::instrument]
pub async fn find_oldest_user(pool: &Pool<MySql>) -> Result<Option<User>> {
    debug!("查找最早的用户");
    let oldest_user = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at ASC LIMIT 1")
        .fetch_optional(pool)
        .await?;
    
    if oldest_user.is_some() {
        debug!("找到最早的用户");
    } else {
        debug!("未找到用户");
    }
    Ok(oldest_user)
}

// 创建 profile 表
#[tracing::instrument]
pub async fn create_profile_table(pool: &Pool<MySql>) -> Result<()> {
    info!("开始创建 profile 表");
    sqlx::query(crate::models::CREATE_PROFILE_TABLE_SQL).execute(pool).await?;
    info!("profile 表创建成功");
    Ok(())
}

// 查询所有 profiles
#[tracing::instrument]
pub async fn select_all_profiles(pool: &Pool<MySql>) -> Result<Vec<crate::models::Profile>> {
    debug!("开始查询所有 profiles");
    let profiles = sqlx::query_as::<_, crate::models::Profile>(crate::models::SELECT_ALL_PROFILES_SQL)
        .fetch_all(pool)
        .await?;
    debug!("查询到 {} 个 profiles", profiles.len());
    Ok(profiles)
}

// 根据 user_id 查询 profile
#[tracing::instrument]
pub async fn select_profile_by_user_id(pool: &Pool<MySql>, user_id: u64) -> Result<Option<crate::models::Profile>> {
    debug!("根据 user_id 查询 profile - user_id: {}", user_id);
    let profile = sqlx::query_as::<_, crate::models::Profile>(crate::models::SELECT_PROFILE_BY_USER_ID_SQL)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if profile.is_some() {
        debug!("找到 profile - user_id: {}", user_id);
    } else {
        debug!("未找到 profile - user_id: {}", user_id);
    }
    Ok(profile)
}