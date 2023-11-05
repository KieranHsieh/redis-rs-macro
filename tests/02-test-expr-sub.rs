use redis_rs_macro::redis;
use redis_test::{MockRedisConnection, MockCmd};


#[test]
fn test_expr_sub() {
    let my_val = 2;
    let mut con = MockRedisConnection::new(vec![
        MockCmd::new(redis!(SET foo {my_val}), Ok("")),
        MockCmd::new(redis!(GET foo), Ok(2)),
        MockCmd::new(redis!(SET bar {my_val + 1}), Ok("")),
        MockCmd::new(redis!(GET bar), Ok(3))
    ]);
    redis!(SET foo {my_val}).execute(&mut con);
    assert_eq!(redis!(GET foo).query(&mut con), Ok(2));
    redis!(SET bar {my_val + 1}).execute(&mut con);
    assert_eq!(redis!(GET bar).query(&mut con), Ok(3));
}