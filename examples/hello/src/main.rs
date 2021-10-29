#[macro_use]
extern crate lazy_static;

use std::{
    sync::{mpsc, Arc, RwLock},
    thread::{self, sleep},
    time::Duration,
};

use xq_notification::notification::{NotificationManager};

use chrono::{self, DateTime, Local, TimeZone};

lazy_static! {
    static ref COUNT_TEST: Arc<RwLock<u64>> = Arc::new(RwLock::new(0));
}

fn main() {
    println!("hello world");

    // system_channel();
    test_notification();

    loop {
        sleep(Duration::from_millis(100));
    }
}

fn system_channel() {
    // 十万次, 40 ms
    let (s, r) = mpsc::channel();
    let max = 100000;
    thread::spawn(move || loop {
        match r.recv() {
            Ok(_) => {
                let mut v = COUNT_TEST.write().unwrap();
                *v += 1;
                if *v == max {
                    println!("recv done");
                    log_time();
                }
            }
            Err(e) => {
                println!("recv error: {}", e);
                break;
            }
        }
    });

    println!("start send: ");
    log_time();
    for i in 0..max {
        let v = serde_json::Value::String(format!("{}", i));
        match s.send(v) {
            Ok(_) => {}
            Err(e) => {
                println!("send error: {}", e);
            }
        }
    }

    println!("end send: ");
    log_time();
    return;
}

fn test_notification() {
    /*
    发送十万次(不同环境，性能可能存在不一样)
    当接收地方只有一个: 120 ms 左右
    当接收地方 10 个: 2.3 s 左右
    当接收地方 100 个: 19.5 s 左右

    发送十万次，接收地方100个
    当不同通知id存在 1000 个(准确是1001，有一个是循环十万次发送的): 20.8 s 左右(多1.3s左右)
    当不同通知id存在 10000 个: 76 s 左右

    因为内部采取的是 hashmap + vec 所以，不同通知 id 数量，还有相同通知监听数量，都会影响速度
     */

    let max = 100000;

    for _ in 0..10 {
        thread::spawn(move || match NotificationManager::observe("test") {
            Some(r) => loop {
                match r.recv() {
                    Ok(_) => {
                        let mut v = COUNT_TEST.write().unwrap();
                        *v += 1;
                        if *v == max {
                            println!("recv done");
                            log_time();
                        }
                    }
                    Err(e) => {
                        println!("error: {}", e);
                        break;
                    }
                }
            },
            None => {}
        });
    }

    // for i in 0..1000 {
    //     let key = i.to_string();
    //     thread::spawn(move || match NotificationManager::observe(&key) {
    //         Some(r) => loop {
    //             match r.recv() {
    //                 Ok(_) => {}
    //                 Err(e) => {
    //                     println!("error: {}", e);
    //                     break;
    //                 }
    //             }
    //         },
    //         None => {}
    //     });
    // }

    println!("start send: ");
    log_time();

    for i in 0..max {
        NotificationManager::publish("test", serde_json::Value::String(format!("{}", i)));
    }

    println!("end send: ");
    log_time();
}

fn log_time() {
    let now: DateTime<Local> = Local::now();
    let mills: i64 = now.timestamp_millis();
    let dt: DateTime<Local> = Local.timestamp_millis(mills);
    println!("{}", dt);
}
