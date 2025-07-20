# Deadpool for libSQL [![Rust 1.81+](https://img.shields.io/badge/rustc-1.81+-lightgray.svg "Rust 1.81+")](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0.html)

Deadpool is a dead simple async pool for connections and objects of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool) manager for [`libSQL`](https://github.com/tursodatabase/libsql/tree/main/libsql) and provides a wrapper that ensures correct use of the connection inside a separate thread.

## Features

| Feature          | Description                                                           | Extra dependencies               | Default |
| ---------------- | --------------------------------------------------------------------- | -------------------------------- | ------- |
| `core`           | Enable core libSQL functionality                                     | `libsql/core`                    | yes     |
| `remote`         | Enable remote database connections                                   | `libsql/remote`                  | yes     |
| `replication`    | Enable database replication support                                  | `libsql/replication`             | no      |
| `sync`           | Enable synchronous operations                                        | `libsql/sync`                    | no      |
| `rt_tokio_1`     | Enable support for [tokio](https://crates.io/crates/tokio) crate     | `deadpool/rt_tokio_1`            | yes     |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/async-std) crate | `deadpool/rt_async-std_1`        | no      |
| `serde`          | Enable support for [serde](https://crates.io/crates/serde) crate     | `deadpool/serde`, `serde/derive` | no      |

**Important:** `async-std` support is currently limited to the
`async-std` specific timeout function. You still need to enable
the `tokio1` feature of `async-std` in order to use this crate
with `async-std`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deadpool-libsql = "0.1"
libsql = "0.9"
tokio = { version = "1.0", features = ["full"] }
```

### Example

```rust,no_run
use deadpool_libsql::{Manager, Pool, PoolConfig, Runtime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database = libsql::Builder::new_local(":memory:")
        .build()
        .await?;
    
    let manager = Manager::from_database(database);
    let pool = Pool::builder(manager)
        .config(PoolConfig::default())
        .runtime(Runtime::Tokio1)
        .build()?;
    
    let conn = pool.get().await?;
    
    conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", ())
        .await?;
    
    conn.execute("INSERT INTO users (name) VALUES (?)", &["Alice"])
        .await?;
    
    let mut rows = conn.query("SELECT id, name FROM users", ()).await?;
    while let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        println!("User: {} - {}", id, name);
    }
    
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
