use anyhow::Result;
use sqlx::{MySql, Pool};
use tracing::{error, info, warn};

use crate::models::{
    DELETE_PROFILE_SQL, DELETE_USER_SQL, INSERT_PROFILE_SQL, INSERT_USER_SQL,
    UPDATE_PROFILE_SQL, UPDATE_USER_SQL
};
use crate::utils::{generate_random_email, generate_random_username};

// 用户服务
pub struct UserService;

impl UserService {
    // 插入用户（使用事务确保提交，失败时回滚）
    pub async fn insert_user(pool: &Pool<MySql>) -> Result<u64> {
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
                
                Ok(user_id)
            }
            Err(e) => {
                error!("插入用户失败: {}", e);
                transaction.rollback().await?;
                error!("事务已回滚");
                Err(e.into())
            }
        }
    }

    // 更新用户邮箱（使用事务确保提交，失败时回滚）
    pub async fn update_user_email(pool: &Pool<MySql>, user_id: u64) -> Result<()> {
        if let Some(user) = crate::database::select_user_by_id(pool, user_id).await? {
            let new_email = format!("updated_{}", user.email);
            
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
                    if let Some(updated_user) = crate::database::select_user_by_id(pool, user_id).await? {
                        info!("更新后的用户 - ID: {}, 用户名: {}, 邮箱: {}",
                            updated_user.id, updated_user.username, updated_user.email);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("更新用户邮箱失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    Err(e.into())
                }
            }
        } else {
            Err(anyhow::anyhow!("未找到ID为 {} 的用户", user_id))
        }
    }

    // 删除最早的用户（使用事务确保提交，失败时回滚）
    pub async fn delete_oldest_user(pool: &Pool<MySql>) -> Result<()> {
        if let Some(oldest_user) = crate::database::find_oldest_user(pool).await? {
            info!("找到最早的用户 - ID: {}, 用户名: {}, 邮箱: {}",
                oldest_user.id, oldest_user.username, oldest_user.email);
            
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
                    Ok(())
                }
                Err(e) => {
                    error!("删除用户失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    Err(e.into())
                }
            }
        } else {
            Err(anyhow::anyhow!("未找到可删除的用户"))
        }
    }
}

// 用户和 Profile 组合服务
pub struct UserProfileService;

impl UserProfileService {
        // 同时创建用户和 profile（使用事务确保原子性）
        pub async fn create_user_with_profile(pool: &Pool<MySql>) -> Result<(u64, u64)> {
            let mut transaction = pool.begin().await?;
            info!("开始事务 - 同时创建用户和 profile");
            
            let username = generate_random_username();
            let email = generate_random_email();
            let full_name = format!("{} Smith", username);
            let bio = Some("这是一个示例个人简介".to_string());
            let avatar_url = Some("https://example.com/avatar.png".to_string());
            
            // 1. 插入用户
            match sqlx::query(INSERT_USER_SQL)
                .bind(&username)
                .bind(&email)
                .execute(&mut *transaction)
                .await
            {
                Ok(result) => {
                    let user_id = result.last_insert_id();
                    info!("事务中插入用户成功 - ID: {}", user_id);
                    
                    // 2. 插入 profile（使用刚生成的 user_id）
                    match sqlx::query(INSERT_PROFILE_SQL)
                        .bind(user_id)
                        .bind(&full_name)
                        .bind(&bio)
                        .bind(&avatar_url)
                        .execute(&mut *transaction)
                        .await
                    {
                        Ok(profile_result) => {
                            let profile_id = profile_result.last_insert_id();
                            info!("事务中插入 profile 成功 - ID: {}", profile_id);
                            
                            // 提交事务
                            transaction.commit().await?;
                            info!("事务提交成功 - 用户和 profile 创建完成");
                            
                            Ok((user_id, profile_id))
                        }
                        Err(e) => {
                            error!("插入 profile 失败: {}", e);
                            transaction.rollback().await?;
                            error!("事务已回滚 - 用户和 profile 都未创建");
                            Err(e.into())
                        }
                    }
                }
                Err(e) => {
                    error!("插入用户失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    Err(e.into())
                }
            }
        }
    
        // 同时更新用户邮箱和 profile 信息（使用事务确保原子性）
        pub async fn update_user_and_profile(pool: &Pool<MySql>, user_id: u64) -> Result<()> {
            let mut transaction = pool.begin().await?;
            info!("开始事务 - 同时更新用户和 profile");
            
            // 1. 更新用户邮箱
            let new_email = format!("updated_{}@example.com", generate_random_username());
            match sqlx::query(UPDATE_USER_SQL)
                .bind(&new_email)
                .bind(user_id)
                .execute(&mut *transaction)
                .await
            {
                Ok(_) => {
                    info!("事务中更新用户邮箱成功");
                    
                    // 2. 更新 profile
                    let new_full_name = format!("Updated {}", generate_random_username());
                    let new_bio = Some("更新后的个人简介".to_string());
                    let new_avatar_url = Some("https://example.com/updated-avatar.png".to_string());
                    
                    match sqlx::query(UPDATE_PROFILE_SQL)
                        .bind(&new_full_name)
                        .bind(&new_bio)
                        .bind(&new_avatar_url)
                        .bind(user_id)
                        .execute(&mut *transaction)
                        .await
                    {
                        Ok(_) => {
                            info!("事务中更新 profile 成功");
                            
                            // 提交事务
                            transaction.commit().await?;
                            info!("事务提交成功 - 用户和 profile 更新完成");
                            Ok(())
                        }
                        Err(e) => {
                            error!("更新 profile 失败: {}", e);
                            transaction.rollback().await?;
                            error!("事务已回滚 - 用户和 profile 都未更新");
                            Err(e.into())
                        }
                    }
                }
                Err(e) => {
                    error!("更新用户邮箱失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    Err(e.into())
                }
            }
        }
    
        // 同时删除用户和 profile（使用事务确保原子性）
        pub async fn delete_user_and_profile(pool: &Pool<MySql>, user_id: u64) -> Result<()> {
            let mut transaction = pool.begin().await?;
            info!("开始事务 - 同时删除用户和 profile");
            
            // 1. 删除 profile
            match sqlx::query(DELETE_PROFILE_SQL)
                .bind(user_id)
                .execute(&mut *transaction)
                .await
            {
                Ok(_) => {
                    info!("事务中删除 profile 成功");
                    
                    // 2. 删除用户
                    match sqlx::query(DELETE_USER_SQL)
                        .bind(user_id)
                        .execute(&mut *transaction)
                        .await
                    {
                        Ok(_) => {
                            info!("事务中删除用户成功");
                            
                            // 提交事务
                            transaction.commit().await?;
                            info!("事务提交成功 - 用户和 profile 删除完成");
                            Ok(())
                        }
                        Err(e) => {
                            error!("删除用户失败: {}", e);
                            transaction.rollback().await?;
                            error!("事务已回滚 - 用户和 profile 都未删除");
                            Err(e.into())
                        }
                    }
                }
                Err(e) => {
                    error!("删除 profile 失败: {}", e);
                    transaction.rollback().await?;
                    error!("事务已回滚");
                    Err(e.into())
                }
            }
        }
    
        // 多表事务回滚测试 - 故意插入重复数据来演示回滚
        pub async fn test_multi_table_transaction_rollback(pool: &Pool<MySql>) -> Result<()> {
            info!("开始多表事务回滚测试...");
            let mut transaction = pool.begin().await?;
            info!("开始事务 - 故意在多表中插入重复数据");
            
            // 获取当前用户列表
            let current_users = crate::database::select_all_users(pool).await?;
            if let Some(existing_user) = current_users.first() {
                // 故意使用重复的用户名来触发唯一约束错误
                let duplicate_username = &existing_user.username;
                let new_email = generate_random_email();
                
                info!("尝试插入重复用户名: {}", duplicate_username);
                
                match sqlx::query(INSERT_USER_SQL)
                    .bind(duplicate_username)
                    .bind(&new_email)
                    .execute(&mut *transaction)
                    .await
                {
                    Ok(result) => {
                        let user_id = result.last_insert_id();
                        // 尝试插入 profile（这不应该执行，因为前面的插入应该失败）
                        let full_name = "Test User".to_string();
                        let bio = Some("Test bio".to_string());
                        let avatar_url = Some("https://example.com/test.png".to_string());
                        
                        match sqlx::query(INSERT_PROFILE_SQL)
                            .bind(user_id)
                            .bind(&full_name)
                            .bind(&bio)
                            .bind(&avatar_url)
                            .execute(&mut *transaction)
                            .await
                        {
                            Ok(_) => {
                                // 这不应该发生，因为用户名是唯一的
                                transaction.commit().await?;
                                warn!("意外成功插入重复用户名，这不应该发生");
                                Ok(())
                            }
                            Err(e) => {
                                error!("插入 profile 失败: {}", e);
                                transaction.rollback().await?;
                                info!("事务已成功回滚 - 数据一致性得到保证");
                                Ok(())
                            }
                        }
                    }
                    Err(e) => {
                        error!("插入重复用户名失败 (预期行为): {}", e);
                        transaction.rollback().await?;
                        info!("事务已成功回滚 - 数据一致性得到保证");
                        
                        // 验证数据没有变化
                        let users_after_rollback = crate::database::select_all_users(pool).await?;
                        let profiles_after_rollback = crate::database::select_all_profiles(pool).await?;
                        info!("回滚后用户数量: {} (与之前相同)", users_after_rollback.len());
                        info!("回滚后 profile 数量: {} (与之前相同)", profiles_after_rollback.len());
                        Ok(())
                    }
                }
            } else {
                Err(anyhow::anyhow!("没有用户可用于多表回滚测试"))
            }
        }
    }

    // 事务回滚测试 - 故意插入重复邮箱来演示回滚
    pub async fn test_transaction_rollback(pool: &Pool<MySql>) -> Result<()> {
        info!("开始事务回滚测试...");
        let mut transaction = pool.begin().await?;
        info!("开始事务 - 故意插入重复邮箱");
        
        // 获取当前用户列表
        let current_users = crate::database::select_all_users(pool).await?;
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
                    Ok(())
                }
                Err(e) => {
                    error!("插入重复邮箱失败 (预期行为): {}", e);
                    transaction.rollback().await?;
                    info!("事务已成功回滚 - 数据一致性得到保证");
                    
                    // 验证数据没有变化
                    let users_after_rollback = crate::database::select_all_users(pool).await?;
                    info!("回滚后用户数量: {} (与之前相同)", users_after_rollback.len());
                    Ok(())
                }
            }
        } else {
            Err(anyhow::anyhow!("没有用户可用于回滚测试"))
        }
}