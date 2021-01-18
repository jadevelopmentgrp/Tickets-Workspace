use async_trait::async_trait;

use super::ShardManager;
use super::Options;

use crate::gateway::{Shard, Identify, ShardInfo};

use model::user::{StatusUpdate, ActivityType, StatusType};

use std::sync::Arc;

use std::collections::HashMap;

use cache::PostgresCache;

use tokio::sync::mpsc;
use tokio::time::delay_for;
use std::time::Duration;
use deadpool_redis::Pool;
use crate::config::Config;
use crate::event_forwarding::HttpEventForwarder;
use crate::gateway::event_forwarding::EventForwarder;

pub struct PublicShardManager<T: EventForwarder> {
    config: Arc<Config>,
    shards: HashMap<u16, Arc<Shard<T>>>,
}

#[cfg(not(feature = "whitelabel"))]
impl<T: EventForwarder> PublicShardManager<T> {
    pub async fn new(
        config: Arc<Config>,
        options: Options,
        cache: Arc<PostgresCache>,
        redis: Arc<Pool>,
        ready_tx: mpsc::Sender<u16>,
        event_forwarder: Arc<T>,
    ) -> Self {
        let mut sm = PublicShardManager {
            config,
            shards: HashMap::new(),
        };

        for i in options.shard_count.lowest..options.shard_count.highest {
            let shard_info = ShardInfo::new(i, options.shard_count.total);
            let status = StatusUpdate::new(ActivityType::Listening, "/help | /setup".to_owned(), StatusType::Online);
            let identify = Identify::new(options.token.clone().into_string(), None, shard_info, Some(status), super::get_intents());

            let shard = Shard::new(
                sm.config.clone(),
                identify,
                options.large_sharding_buckets,
                Arc::clone(&cache),
                Arc::clone(&redis),
                options.user_id,
                Some(ready_tx.clone()),
                Arc::clone(&event_forwarder),
            );

            sm.shards.insert(i, shard);
        }

        sm
    }
}

#[async_trait]
impl<T: EventForwarder> ShardManager for PublicShardManager<T> {
    async fn connect(self: Arc<Self>) {
        for (_, shard) in self.shards.iter() {
            let shard = Arc::clone(&shard);

            tokio::spawn(async move {
                loop {
                    let shard = Arc::clone(&shard);
                    shard.log("Starting...");

                    match Arc::clone(&shard).connect().await {
                        Ok(()) => shard.log("Exited with Ok"),
                        Err(e) => shard.log_err("Exited with error", &e),
                    }

                    delay_for(Duration::from_millis(500)).await;
                }
            });
        }
    }
}
