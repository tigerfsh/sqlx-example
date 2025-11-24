use anyhow::Result;
use tracing::{Level, debug, error, info, warn};
use tracing_subscriber;

// 导入模块
mod models;
mod database;
mod services;
mod utils;

use crate::database::{create_pool, create_table, select_all_users, select_user_by_id};
use crate::services::{UserService, UserProfileService};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("启动 SQLx MySQL 示例程序");

    // 1. 创建数据库连接池
    let pool = create_pool().await?;

    // 2. 创建表
    create_table(&pool).await?;
    crate::database::create_profile_table(&pool).await?;
    info!("用户表和 profile 表创建/检查完成");

    // 3. 插入数据（使用事务确保提交，失败时回滚）
    let user_id = UserService::insert_user(&pool).await?;
    info!("插入用户成功，ID: {}", user_id);

    // 4. 查询所有数据
    let users = select_all_users(&pool).await?;
    info!("查询到 {} 个用户", users.len());
    for user in &users {
        debug!(
            "用户详情 - ID: {}, 用户名: {}, 邮箱: {}, 创建时间: {}, 更新时间: {}",
            user.id, user.username, user.email, user.created_at, user.updated_at
        );
    }

    // 5. 根据ID查询数据
    if let Some(user) = select_user_by_id(&pool, user_id).await? {
        info!(
            "根据ID查询用户成功 - ID: {}, 用户名: {}, 邮箱: {}",
            user.id, user.username, user.email
        );
    } else {
        warn!("未找到ID为 {} 的用户", user_id);
    }

    // 6. 更新操作 - 只更新邮箱（使用事务确保提交，失败时回滚）
    if let Err(e) = UserService::update_user_email(&pool, user_id).await {
        error!("更新用户失败: {}", e);
    }

    // 7. 删除操作 - 删除最早写入的用户（使用事务确保提交，失败时回滚）
    if let Err(e) = UserService::delete_oldest_user(&pool).await {
        warn!("删除用户失败: {}", e);
    }

    // 8. 多表事务操作演示 - 同时创建用户和 profile
    info!("开始多表事务操作演示...");
    match UserProfileService::create_user_with_profile(&pool).await {
        Ok((user_id, profile_id)) => {
            info!("多表事务创建成功 - 用户ID: {}, Profile ID: {}", user_id, profile_id);
            
            // 验证创建的数据
            if let Some(user) = select_user_by_id(&pool, user_id).await? {
                info!("创建的用户 - ID: {}, 用户名: {}, 邮箱: {}",
                    user.id, user.username, user.email);
            }
            
            if let Some(profile) = crate::database::select_profile_by_user_id(&pool, user_id).await? {
                info!("创建的 Profile - ID: {}, 用户ID: {}, 全名: {}, 简介: {:?}",
                    profile.id, profile.user_id, profile.full_name, profile.bio);
            }
        }
        Err(e) => {
            error!("多表事务创建失败: {}", e);
        }
    }

    // 9. 多表事务更新演示
    if let Some(user) = crate::database::select_all_users(&pool).await?.first() {
        if let Err(e) = UserProfileService::update_user_and_profile(&pool, user.id).await {
            warn!("多表事务更新失败: {}", e);
        }
    }

    // 10. 事务回滚测试 - 故意插入重复数据来演示回滚
    if let Err(e) = UserProfileService::test_multi_table_transaction_rollback(&pool).await {
        warn!("多表事务回滚测试失败: {}", e);
    }

    // 11. 最终验证 - 查询所有数据确认数据持久化
    info!("最终验证 - 查询数据库中的所有用户:");
    let final_users = select_all_users(&pool).await?;
    info!("数据库中实际存在的用户数量: {}", final_users.len());
    for user in &final_users {
        info!(
            "最终用户数据 - ID: {}, 用户名: {}, 邮箱: {}",
            user.id, user.username, user.email
        );
    }

    info!("最终验证 - 查询数据库中的所有 profiles:");
    let final_profiles = crate::database::select_all_profiles(&pool).await?;
    info!("数据库中实际存在的 profile 数量: {}", final_profiles.len());
    for profile in &final_profiles {
        info!(
            "最终 Profile 数据 - ID: {}, 用户ID: {}, 全名: {}, 简介: {:?}",
            profile.id, profile.user_id, profile.full_name, profile.bio
        );
    }

    info!("SQLx MySQL 示例程序执行完成 - 所有事务操作（包括多表事务和回滚测试）已完成");
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
