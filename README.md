# xq-notification-rs

rust 通知封装


# 使用介绍

Cargo.toml 

```
serde_json = "^1.0"
xq_notification = "*"
```

示例代码

```rust
use std::{thread};
use xq_notification::notification::NotificationManager;
use serde_json;

fn main() {
    // 第一个通知
    thread::spawn(move || match NotificationManager::observe("test") {
        Some(r) => loop {
            match r.recv() {
                Ok(_) => {
                    // 接收到通知
                }
                Err(_) => {
                    break;
                }
            }
        },
        None => {}
    }); 

    // 第二个通知
    thread::spawn(move || match NotificationManager::observe("test") {
        Some(r) => loop {
            match r.recv() {
                Ok(_) => {
                    // 接收到通知
                }
                Err(_) => {
                    break;
                }
            }
        },
        None => {}
    });

    // 发送通知
    for i in 0..10000 {
        NotificationManager::publish("test", serde_json::Value::String(format!("{}", i)));
    }

}
```
