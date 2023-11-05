use redis::Value;
use redis_rs_macro::redis;
use redis_test::{MockCmd, MockRedisConnection};

#[test]
fn test_base_usage_str() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo "hello there"), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("hello there")),
    ]);

    redis!(SET foo "hello there").execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"hello there".as_ref().into()))
    );
}

#[test]
fn test_base_usage_bool() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo true), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("true")),
    ]);

    redis!(SET foo true).execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"true".as_ref().into()))
    );
}

#[test]
fn test_base_usage_int() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo 42), Ok("")),
        MockCmd::new(redis!(GET foo), Ok(42)),
    ]);

    redis!(SET foo 42).execute(&mut conn);
    assert_eq!(redis!(GET foo).query(&mut conn), Ok(42));
}

#[test]
fn test_base_usage_float() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo 42.2), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("42.2")),
    ]);

    redis!(SET foo 42.2).execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"42.2".as_ref().into()))
    );
}

#[test]
fn test_base_usage_plus() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo +), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("+")),
    ]);

    redis!(SET foo +).execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"+".as_ref().into()))
    );
}

#[test]
fn test_base_usage_minus() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo -), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("-")),
    ]);

    redis!(SET foo -).execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"-".as_ref().into()))
    );
}

#[test]
fn test_base_usage_star() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo *), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("*")),
    ]);

    redis!(SET foo *).execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"*".as_ref().into()))
    );
}

#[test]
fn test_base_usage_dollar() {
    let mut conn = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo $), Ok("")),
        MockCmd::new(redis!(GET foo), Ok("$")),
    ]);

    redis!(SET foo $).execute(&mut conn);
    assert_eq!(
        redis!(GET foo).query(&mut conn),
        Ok(Value::Data(b"$".as_ref().into()))
    );
}
