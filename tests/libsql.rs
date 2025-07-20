use deadpool_libsql::{Manager, Pool, PoolConfig, Runtime};

async fn create_pool() -> Pool {
    let database = libsql::Builder::new_local(":memory:")
        .build()
        .await
        .unwrap();
    let manager = Manager::from_database(database);
    Pool::builder(manager)
        .config(PoolConfig::default())
        .runtime(Runtime::Tokio1)
        .build()
        .unwrap()
}

#[tokio::test]
async fn basic() {
    let pool = create_pool().await;
    let conn = pool.get().await.unwrap();

    // Create a table
    conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)", ())
        .await
        .unwrap();

    // Insert a row
    conn.execute("INSERT INTO test (value) VALUES (?)", &["Hello"])
        .await
        .unwrap();

    // Query the row
    let mut rows = conn.query("SELECT id, value FROM test", ()).await.unwrap();
    let row = rows.next().await.unwrap().unwrap();
    assert_eq!(row.column_count(), 2);
    assert_eq!(row.column_name(0), Some("id"));
    assert_eq!(row.get::<i64>(0).unwrap(), 1);
    assert_eq!(row.column_name(1), Some("value"));
    assert_eq!(row.get::<String>(1).unwrap(), "Hello");
}

#[tokio::test]
async fn multiple_connections() {
    let pool = create_pool().await;

    // Get multiple connections simultaneously
    let conn1 = pool.get().await.unwrap();
    let conn2 = pool.get().await.unwrap();

    // Both connections should work independently
    conn1
        .execute("CREATE TABLE test1 (id INTEGER)", ())
        .await
        .unwrap();
    conn2
        .execute("CREATE TABLE test2 (id INTEGER)", ())
        .await
        .unwrap();

    conn1
        .execute("INSERT INTO test1 (id) VALUES (1)", ())
        .await
        .unwrap();
    conn2
        .execute("INSERT INTO test2 (id) VALUES (2)", ())
        .await
        .unwrap();

    // Verify data in each connection
    let mut rows1 = conn1.query("SELECT id FROM test1", ()).await.unwrap();
    let row1 = rows1.next().await.unwrap().unwrap();
    assert_eq!(row1.get::<i64>(0).unwrap(), 1);

    let mut rows2 = conn2.query("SELECT id FROM test2", ()).await.unwrap();
    let row2 = rows2.next().await.unwrap().unwrap();
    assert_eq!(row2.get::<i64>(0).unwrap(), 2);
}

#[tokio::test]
async fn connection_recycling() {
    let pool = create_pool().await;

    // Get and return a connection multiple times
    for _ in 0..5 {
        let conn = pool.get().await.unwrap();
        let mut _rows = conn.query("SELECT 1", ()).await.unwrap();
        // Connection is automatically returned to pool when dropped
        drop(conn);
    }
}
