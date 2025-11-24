# SQLx MySQL 示例

这是一个使用 Rust SQLx 库操作 MySQL 数据库的完整示例，包含建表、增删改查等基本操作。

## 项目结构

```
sqlx-example/
├── Cargo.toml      # 项目依赖配置
├── src/
│   └── main.rs     # 主要代码文件
└── README.md       # 项目说明
```

## 依赖说明

- `sqlx`: 异步 SQL 数据库工具包，支持 MySQL
- `tokio`: 异步运行时
- `serde`: 序列化/反序列化库
- `anyhow`: 错误处理
- `chrono`: 日期时间处理
- `tracing`: 结构化日志系统
- `tracing-subscriber`: 日志订阅器

## 数据库表结构

示例创建了一个 `users` 表，包含以下字段：

- `id`: BIGINT UNSIGNED AUTO_INCREMENT 主键
- `username`: VARCHAR(50) 用户名，唯一
- `email`: VARCHAR(100) 邮箱，唯一
- `created_at`: TIMESTAMP 创建时间
- `updated_at`: TIMESTAMP 更新时间

## 功能特性

1. **建表操作**: 自动创建用户表
2. **插入数据**: 添加新用户
3. **查询数据**:
   - 查询所有用户
   - 根据ID查询用户
4. **更新数据**: 修改用户信息
5. **删除数据**: 删除指定用户
6. **结构化日志**: 使用 tracing 库提供详细的执行日志

## 使用方法

### 1. 配置数据库连接

程序会自动使用环境变量 `DATABASE_URL`，如果没有设置则使用默认值：

```rust
let database_url = "mysql://root:password@localhost:3306/testdb?ssl-mode=disabled";
```

### 2. 运行程序

```bash
cargo run
```

### 3. 环境变量配置（推荐）

通过环境变量设置数据库连接：

```bash
export DATABASE_URL="mysql://用户名:密码@主机:端口/数据库名"
cargo run
```

### 4. 数据库连接问题处理

如果遇到 TLS/SSL 握手失败，程序会自动重试禁用 SSL 的连接：

```rust
let database_url_no_ssl = format!("{}?ssl-mode=disabled", database_url);
```

## 代码说明

### 数据结构

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: u64,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### 主要函数

- `create_table()`: 创建用户表
- `insert_user()`: 插入用户数据
- `select_all_users()`: 查询所有用户
- `select_user_by_id()`: 根据ID查询用户
- `update_user()`: 更新用户信息
- `delete_user()`: 删除用户

## 注意事项

1. 确保 MySQL 服务正在运行
2. 数据库需要提前创建
3. 根据实际情况修改数据库连接信息
4. 表结构会自动创建，无需手动建表

## 日志系统

项目使用 `tracing` 库提供结构化日志，包含以下日志级别：

- **INFO**: 主要操作信息（程序启动、数据库连接、CRUD操作结果）
- **DEBUG**: 详细的操作过程和中间结果
- **WARN**: 警告信息（如未找到用户等）
- **ERROR**: 错误信息（数据库连接失败、SQL执行错误等）

### 日志配置

日志系统在程序启动时自动初始化：
```rust
tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .with_target(false)
    .init();
```

### 函数级日志

所有数据库操作函数都使用 `#[instrument]` 宏自动记录：
- 函数调用参数
- 执行开始和结束时间
- 执行结果

## 扩展建议

1. 添加事务处理
2. 实现分页查询
3. 添加数据验证
4. 实现更复杂的查询条件
5. 添加索引优化查询性能
6. 添加日志文件输出
7. 实现日志轮转
8. 添加性能监控指标

## 相关链接

- [SQLx 文档](https://docs.rs/sqlx)
- [MySQL 文档](https://dev.mysql.com/doc/)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)