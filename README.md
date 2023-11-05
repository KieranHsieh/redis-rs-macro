# Redis Macro

A simple macro to make creating Redis commands (From [redis-rs](https://github.com/redis-rs/redis-rs.git)) more readable.

# Cargo.toml
```toml
[dependencies]
redis_rs_macro = "1"
redis = "0.23"
```

# Usage
```rust
use redis_rs_macro::redis;

fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    // redis::cmd("SET").arg("my_key").arg("1").execute(&mut con);
    redis!(SET my_key 1).execute(&mut con);

    // redis::cmd("GET").arg("my_key").query(&mut con)?;
    let ret: i32 = redis!(GET my_key).query(&mut con)?;
    assert_eq!(ret, 1);
    Ok(())
}
```