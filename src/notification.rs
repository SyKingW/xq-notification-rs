use std::{collections::HashMap, sync::{mpsc, Arc, RwLock}, thread::{self, sleep}, time::Duration};

use lazy_static;

use log::error;
use uuid::Uuid;

use serde_json::Value;

lazy_static! {
    static ref XQ_NOTIFICATION_MANAGER: Arc<RwLock<NotificationManager>> =
        Arc::new(RwLock::new(NotificationManager::new()));
}

/// 接收和发送的类型
/// 这个要想一下怎么改成泛型
pub type NotificationType = Value;

/// 还是有一个问题...这样改之后，没办法知道外部不用这个通知了，所以就会一直持有 sender
pub struct NotificationManager {
    sender_map: HashMap<String, Vec<NotificationObj>>,
}

unsafe impl Send for NotificationManager {}
unsafe impl Sync for NotificationManager {}

#[derive(Debug, Clone)]
struct NotificationObj {
    id: String,
    notification_id: String,
    sender: mpsc::Sender<NotificationType>,
}

impl NotificationManager {
    fn new() -> Self {

        thread::spawn(|| {
            // 30 秒检测一次，是否存在空
            sleep(Duration::from_secs(30));
            match XQ_NOTIFICATION_MANAGER.write() {
                Ok(_) => {},
                Err(_) => {},
            }
        });

        NotificationManager {
            sender_map: HashMap::new(),
        }
    }

    /// 监听通知
    pub fn observe(notification_id: &str) -> Option<mpsc::Receiver<NotificationType>> {
        match XQ_NOTIFICATION_MANAGER.write() {
            Ok(mut manager) => {
                let (s, r) = mpsc::channel();
                let id = Uuid::new_v4().to_string();
                let obj = NotificationObj {
                    id,
                    notification_id: notification_id.to_string(),
                    sender: s,
                };

                match manager.sender_map.get_mut(notification_id) {
                    Some(senders) => {
                        senders.push(obj);
                    }
                    None => {
                        manager
                            .sender_map
                            .insert(notification_id.to_string(), vec![obj]);
                    }
                }

                return Some(r);
            }
            Err(e) => {
                error!("observe try_write error: {}", e);
            }
        }

        None
    }

    /// 发送通知
    pub fn publish(notification_id: &str, msg: NotificationType) -> bool {
        match XQ_NOTIFICATION_MANAGER.write() {
            Ok(mut manager) => {
                return manager.publish_s(notification_id, msg);
            },
            Err(e) => {
                error!("publish write error: {}", e);
            }
        }

        false
    }

    fn publish_s(&mut self, notification_id: &str, msg: NotificationType) -> bool {
        match self.sender_map.get_mut(notification_id) {
            Some(senders) => {
                let mut index = 0;

                let mut send_succeed = false;

                while index < senders.len() {
                    let item = &senders[index];
                    if item.notification_id.eq(notification_id) {
                        match item.sender.send(msg.clone()) {
                            Ok(_) => {
                                send_succeed = true;
                            }
                            Err(e) => {
                                if e.to_string().eq("sending on a closed channel") {
                                    senders.remove(index);
                                    continue;
                                } else {
                                    error!("publish error: {}", e);
                                }
                            }
                        }
                    }

                    index += 1;
                }

                if senders.len() == 0 {
                    self.sender_map.remove(notification_id);
                }

                return send_succeed;
            }
            None => {}
        }
        false
    }

    /// 清除某个 通知id 所有的 receiver
    pub fn clear_notification_id(notification_id: &str) {
        match XQ_NOTIFICATION_MANAGER.write() {
            Ok(mut manager) => {
                manager.sender_map.remove(notification_id);
            }
            Err(_) => {}
        }
    }

    /// 根据id，清除某个 receiver
    #[allow(dead_code)]
    fn drop_receiver_from_id(id: &str) {
        match XQ_NOTIFICATION_MANAGER.write() {
            Ok(mut manager) => {
                for (_, value) in manager.sender_map.iter_mut() {
                    let mut index = 0;

                    while index < value.len() {
                        let item = &value[index];

                        if item.id.eq(id) {
                            // 理论上只有一个 id，所以这里找到一个之后，直接返回
                            value.remove(index);
                            return;
                        } else {
                            index += 1;
                        }
                    }
                }

                // manager.sender_map.remove(notification_id);
            }
            Err(_) => {}
        }
    }
}
