use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{Rng, distributions::Alphanumeric, thread_rng};
use rand::{seq::SliceRandom};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};
use std::env;
use tracing::{Level, debug, error, info, instrument, warn};

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
UPDATE users SET email = ? WHERE id = ?
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

    // 1. 创建表
    create_table(&pool).await?;
    info!("用户表创建/检查完成");

    // 2. 插入数据（使用事务确保提交，失败时回滚）
    let user_id = {
        let mut transaction = pool.begin().await?;
        info!("开始事务插入用户");
        
        let username = generate_random_username();
        let email = generate_random_email();
        
        match sqlx::query(INSERT_USER_SQL)
            .bind(&username)
            .bind(&email)
            .execute(&mut *transaction)
            .await
        {
            Ok(result) => {
                let user_id = result.last_insert_id();
                info!("事务中插入用户成功 - ID: {}", user_id);
                
                // 提交事务
                transaction.commit().await?;
                info!("事务提交成功");
                
                user_id
            }
            Err(e) => {
                error!("插入用户失败: {}", e);
                transaction.rollback().await?;
                error!("事务已回滚");
                return Err(e.into());
            }
        }
    };
    info!("插入用户成功，ID: {}", user_id);

    // 3. 查询所有数据
    let users = select_all_users(&pool).await?;
    info!("查询到 {} 个用户", users.len());
    for user in &users {
        debug!(
            "用户详情 - ID: {}, 用户名: {}, 邮箱: {}, 创建时间: {}, 更新时间: {}",
            user.id, user.username, user.email, user.created_at, user.updated_at
        );
    }

    // 4. 根据ID查询数据
    if let Some(user) = select_user_by_id(&pool, user_id).await? {
        info!(
            "根据ID查询用户成功 - ID: {}, 用户名: {}, 邮箱: {}",
            user.id, user.username, user.email
        );
    } else {
        warn!("未找到ID为 {} 的用户", user_id);
    }

    // 5. 更新操作 - 只更新邮箱（使用事务确保提交，失败时回滚）
    if let Some(user) = select_user_by_id(&pool, user_id).await? {
        let new_email = format!("updated_{}", user.email);
        
        {
            let mut transaction = pool.begin().await?;
            info!("开始事务更新用户邮箱");
            
            match sqlx::query(UPDATE_USER_SQL)
                .bind(&new_email)
                .bind(user_id)
                .execute(&mut *transaction)
                .await
            {
                Ok(_) => {
                    transaction.commit().await?;
                    info!("事务提交成功");
                    info!("更新用户邮箱成功 - ID: {}, 新邮箱: {}", user_id, new_email);
                    
                    // 验证更新
                    if let Some(updated_user) = select_user_by_id(&pool, user_id).await? {
                        info!("更新后的用户 - ID: {}, 用户名: {}, 邮箱: {}",
                            updated_user.id, updated_user.username, updated_user.email);
                    }
                }
                Err(e) => {
                    error!("更新用户邮箱失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    return Err(e.into());
                }
            }
        }
    }

    // 6. 删除操作 - 删除最早写入的用户（使用事务确保提交，失败时回滚）
    if let Some(oldest_user) = find_oldest_user(&pool).await? {
        info!("找到最早的用户 - ID: {}, 用户名: {}, 邮箱: {}",
            oldest_user.id, oldest_user.username, oldest_user.email);
        
        {
            let mut transaction = pool.begin().await?;
            info!("开始事务删除用户");
            
            match sqlx::query(DELETE_USER_SQL)
                .bind(oldest_user.id)
                .execute(&mut *transaction)
                .await
            {
                Ok(_) => {
                    transaction.commit().await?;
                    info!("事务提交成功");
                    info!("删除最早用户成功 - ID: {}", oldest_user.id);
                }
                Err(e) => {
                    error!("删除用户失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    return Err(e.into());
                }
            }
        }
    } else {
        warn!("未找到可删除的用户");
    }

    // 7. 事务回滚测试 - 故意插入重复邮箱来演示回滚
    info!("开始事务回滚测试...");
    {
        let mut transaction = pool.begin().await?;
        info!("开始事务 - 故意插入重复邮箱");
        
        // 获取当前用户列表
        let current_users = select_all_users(&pool).await?;
        if let Some(existing_user) = current_users.first() {
            // 故意使用重复的邮箱来触发唯一约束错误
            let duplicate_email = &existing_user.email;
            let new_username = generate_random_username();
            
            info!("尝试插入重复邮箱: {}", duplicate_email);
            
            match sqlx::query(INSERT_USER_SQL)
                .bind(&new_username)
                .bind(duplicate_email)
                .execute(&mut *transaction)
                .await
            {
                Ok(_) => {
                    // 这不应该发生，因为邮箱是唯一的
                    transaction.commit().await?;
                    warn!("意外成功插入重复邮箱，这不应该发生");
                }
                Err(e) => {
                    error!("插入重复邮箱失败 (预期行为): {}", e);
                    transaction.rollback().await?;
                    info!("事务已成功回滚 - 数据一致性得到保证");
                    
                    // 验证数据没有变化
                    let users_after_rollback = select_all_users(&pool).await?;
                    info!("回滚后用户数量: {} (与之前相同)", users_after_rollback.len());
                }
            }
        }
    }

    // 8. 最终验证 - 查询所有数据确认数据持久化
    info!("最终验证 - 查询数据库中的所有用户:");
    let final_users = select_all_users(&pool).await?;
    info!("数据库中实际存在的用户数量: {}", final_users.len());
    for user in &final_users {
        info!(
            "最终用户数据 - ID: {}, 用户名: {}, 邮箱: {}",
            user.id, user.username, user.email
        );
    }

    info!("SQLx MySQL 示例程序执行完成 - 所有事务操作（包括回滚测试）已完成");
    Ok(())
}

// 创建用户表
#[instrument]
async fn create_table(pool: &Pool<MySql>) -> Result<()> {
    info!("开始创建用户表");
    sqlx::query(CREATE_USER_TABLE_SQL).execute(pool).await?;
    info!("用户表创建成功");
    Ok(())
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

fn generate_random_username() -> String {
    let mut rng = thread_rng();
    let username: String = (&mut rng)
        .sample_iter(Alphanumeric)
        .filter(|c| c.is_ascii_alphabetic())
        .map(char::from)
        .take(10)
        .collect();
    username
}

fn generate_random_email() -> String {
    let username = generate_random_username().to_lowercase();
    let domains = ["example.com", "test.com", "mail.com", "demo.org"];

    let mut rng = thread_rng();
    let domain = domains.choose(&mut rng).unwrap_or(&"example.com");
    format!("{}@{}", username, domain)
}

// 查找最早的用户
#[instrument]
async fn find_oldest_user(pool: &Pool<MySql>) -> Result<Option<User>> {
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

// 简单的测试函数
#[tokio::test]
async fn test_basic_operations() -> Result<()> {
    // 这个测试需要实际的数据库连接，所以这里只是演示结构
    // 在实际项目中，你可以使用测试数据库
    info!("测试结构演示");
    Ok(())
}
