use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env;
use tracing::{debug, error, info, instrument, warn, Level};

// 用户表结构
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: u64,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// 创建用户表的SQL
const CREATE_USER_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
"#;

// 插入用户的SQL
const INSERT_USER_SQL: &str = r#"
INSERT INTO users (username, email) VALUES (?, ?)
"#;

// 查询所有用户的SQL
const SELECT_ALL_USERS_SQL: &str = r#"
SELECT id, username, email, created_at, updated_at FROM users
"#;

// 根据ID查询用户的SQL
const SELECT_USER_BY_ID_SQL: &str = r#"
SELECT id, username, email, created_at, updated_at FROM users WHERE id = ?
"#;

// 更新用户的SQL
const UPDATE_USER_SQL: &str = r#"
UPDATE users SET username = ?, email = ? WHERE id = ?
"#;

// 删除用户的SQL
const DELETE_USER_SQL: &str = r#"
DELETE FROM users WHERE id = ?
"#;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("启动 SQLx MySQL 示例程序");

    // 从环境变量获取数据库URL，如果没有设置则使用默认值
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:password@localhost:3306/testdb".to_string());

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

    // 1. 创建表
    create_table(&pool).await?;
    info!("用户表创建/检查完成");

    // 2. 插入数据
    let user_id = insert_user(&pool, "张三", "zhangsan@example.com").await?;
    info!("插入用户成功，ID: {}", user_id);

    // 3. 查询所有数据
    let users = select_all_users(&pool).await?;
    info!("查询到 {} 个用户", users.len());
    for user in &users {
        debug!("用户详情 - ID: {}, 用户名: {}, 邮箱: {}, 创建时间: {}, 更新时间: {}",
            user.id, user.username, user.email, user.created_at, user.updated_at);
    }

    // 4. 根据ID查询数据
    if let Some(user) = select_user_by_id(&pool, user_id).await? {
        info!("根据ID查询用户成功 - ID: {}, 用户名: {}, 邮箱: {}", user.id, user.username, user.email);
    } else {
        warn!("未找到ID为 {} 的用户", user_id);
    }

    info!("SQLx MySQL 示例程序执行完成");
    Ok(())
}


// 创建用户表
#[instrument]
async fn create_table(pool: &Pool<MySql>) -> Result<()> {
    info!("开始创建用户表");
    sqlx::query(CREATE_USER_TABLE_SQL)
        .execute(pool)
        .await?;
    info!("用户表创建成功");
    Ok(())
}

// 插入用户
#[instrument]
async fn insert_user(pool: &Pool<MySql>, username: &str, email: &str) -> Result<u64> {
    info!("开始插入用户 - 用户名: {}, 邮箱: {}", username, email);
    let result = sqlx::query(INSERT_USER_SQL)
        .bind(username)
        .bind(email)
        .execute(pool)
        .await?;

    let user_id = result.last_insert_id();
    info!("用户插入成功 - ID: {}", user_id);
    Ok(user_id)
}

// 查询所有用户
#[instrument]
async fn select_all_users(pool: &Pool<MySql>) -> Result<Vec<User>> {
    debug!("开始查询所有用户");
    let users = sqlx::query_as::<_, User>(SELECT_ALL_USERS_SQL)
        .fetch_all(pool)
        .await?;
    debug!("查询到 {} 个用户", users.len());
    Ok(users)
}

// 根据ID查询用户
#[instrument]
async fn select_user_by_id(pool: &Pool<MySql>, id: u64) -> Result<Option<User>> {
    debug!("根据ID查询用户 - ID: {}", id);
    let user = sqlx::query_as::<_, User>(SELECT_USER_BY_ID_SQL)
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

// 更新用户
#[instrument]
async fn update_user(pool: &Pool<MySql>, id: u64, username: &str, email: &str) -> Result<()> {
    info!("开始更新用户 - ID: {}, 新用户名: {}, 新邮箱: {}", id, username, email);
    sqlx::query(UPDATE_USER_SQL)
        .bind(username)
        .bind(email)
        .bind(id)
        .execute(pool)
        .await?;
    info!("用户更新成功 - ID: {}", id);
    Ok(())
}

// 删除用户
#[instrument]
async fn delete_user(pool: &Pool<MySql>, id: u64) -> Result<()> {
    info!("开始删除用户 - ID: {}", id);
    sqlx::query(DELETE_USER_SQL)
        .bind(id)
        .execute(pool)
        .await?;
    info!("用户删除成功 - ID: {}", id);
    Ok(())
}

// 在事务中删除用户
#[instrument]
async fn delete_user_in_transaction(transaction: &mut sqlx::Transaction<'_, MySql>, id: u64) -> Result<()> {
    info!("在事务中删除用户 - ID: {}", id);
    sqlx::query(DELETE_USER_SQL)
        .bind(id)
        .execute(&mut **transaction)
        .await?;
    info!("事务中用户删除成功 - ID: {}", id);
    Ok(())
}

// 简单的测试函数
#[tokio::test]
async fn test_basic_operations() -> Result<()> {
    // 这个测试需要实际的数据库连接，所以这里只是演示结构
    // 在实际项目中，你可以使用测试数据库
    info!("测试结构演示");
    Ok(())
}
