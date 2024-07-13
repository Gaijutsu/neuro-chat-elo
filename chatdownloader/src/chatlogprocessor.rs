use core::panic;
use log::{debug, info, warn};
use std::time::Instant;
use std::{collections::HashMap, fs};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::_types::clptypes::{ChatPerformance, MetadataTypes, PerformanceType};
use crate::_types::twitchtypes::{ChatLog, Comment};
use crate::twitch_utils::TwitchAPIWrapper;

use crate::leaderboards::get_leaderboards;
use crate::metadata::get_metadata;
use crate::metrics::get_metrics;

pub struct ChatLogProcessor<'a> {
    /*
    Processes the chat logs.

    The class uses the metrics package to extract metrics from the
    chat messages, the metadata package to extract any user-metadata,
    and the leaderboards package to export the metrics / required user
    metadata to the right people
    */
    twitch: &'a TwitchAPIWrapper,
}

impl<'a> ChatLogProcessor<'a> {
    pub fn new(twitch: &'a TwitchAPIWrapper) -> Self {
        Self { twitch }
    }

    pub fn __parse_to_log_struct(&self, chat_log_path: String) -> ChatLog {
        let chat_log_str = fs::read_to_string(chat_log_path).unwrap();
        let chat_log: ChatLog = serde_json::from_str(&chat_log_str).unwrap();
        chat_log
    }

    pub async fn parse_from_log_object(&self, chat_log: ChatLog) -> Vec<ChatPerformance> {
        let start_time = Instant::now();

        let (metrics, _metric_join_handles, metric_sender, metric_receiver) = get_metrics().await;

        let (metadatas, _metadata_join_handles, metadata_sender, metadata_receiver) =
            get_metadata(self.twitch).await;

        info!("Parsing chat log object");
        spawn_chatlog_to_receiver(chat_log.clone(), vec![metric_sender, metadata_sender]);

        let performances = if let Ok(performances) = spawn_user_chat_perforance_thread(
            metrics,
            metric_receiver,
            metadatas,
            metadata_receiver,
        )
        .await
        {
            performances.to_owned()
        } else {
            panic!("Could not get user performances");
        };
        info!(
            "Chat log processing took: {}ms",
            start_time.elapsed().as_millis()
        );
        performances.values().cloned().collect()
    }

    #[allow(dead_code)]
    async fn parse(&self, chat_log_path: String) -> Vec<ChatPerformance> {
        let chat_log = self.__parse_to_log_struct(chat_log_path);
        self.parse_from_log_object(chat_log).await
    }

    /// A function to export the user performances to the leaderboards and save them
    pub fn export_to_leaderboards(performances: Vec<ChatPerformance>) {
        let (_leaderboard_handles, leaderboard_sender) = get_leaderboards();
        for performance in performances {
            match leaderboard_sender.send(performance) {
                Ok(_) => {}
                Err(e) => warn!("Could not send performance to leaderboards: {:?}", e),
            };
        }
    }
}

/// A function to spawn a thread to take a ChatLog and add its comments to a receiver
pub fn spawn_chatlog_to_receiver(
    chat_log: ChatLog,
    senders: Vec<broadcast::Sender<(Comment, u32)>>,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        for (sequence_no, comment) in chat_log.comments.iter().enumerate() {
            for sender in senders.iter() {
                sender.send((comment.clone(), sequence_no as u32)).unwrap();
            }
        }
        debug!("Finished sending chat log to receivers")
    })
}

/// A function to spawn a thread that takes two recievers and processes metrics / metadata from them and updates the user performances
pub fn spawn_user_chat_perforance_thread(
    metrics: HashMap<String, f32>,
    mut metric_receiver: mpsc::Receiver<(String, HashMap<String, f32>)>,
    metadatas: HashMap<String, MetadataTypes>,
    mut metadata_receiver: mpsc::Receiver<(String, HashMap<String, MetadataTypes>)>,
) -> JoinHandle<HashMap<String, ChatPerformance>> {
    tokio::task::spawn(async move {
        let mut user_performances: HashMap<String, ChatPerformance> = HashMap::new();
        loop {
            tokio::select! {
                Some((metric_name, metric_update)) = metric_receiver.recv() => {
                    for (user_id, met_value) in metric_update.iter() {
                        get_performance_or_default(&mut user_performances, user_id, &metrics, &metadatas);
                        if let Some(chat_performance) = user_performances.get_mut(user_id) {
                            if let Some(metric_value) = chat_performance.metrics.get_mut(&metric_name) {
                                *metric_value += met_value;
                                debug!("Updating metric: {} with value: {:?}", metric_name, met_value);
                            }
                        }
                    }
                }
                Some((metadata_name, metadata_update)) = metadata_receiver.recv() => {
                    for (user_id, met_value) in metadata_update.iter() {
                        get_performance_or_default(&mut user_performances, user_id, &metrics, &metadatas);
                        if let Some(chat_performance) = user_performances.get_mut(user_id) {
                            if metadata_name == "basic_info" {
                                let (username, avatar, perf_type) = match met_value.get_basic_info() {
                                    Some((username, avatar, perf_type)) => (username, avatar, perf_type),
                                    None => {warn!("Could not get username and/or url"); ("".to_string(), "".to_string(), PerformanceType::Unknown)}
                                };
                                chat_performance.username = username;
                                chat_performance.avatar = avatar;
                                chat_performance.perf_type = perf_type;
                            } else if let Some(metadata_value) = chat_performance.metadata.get_mut(&metadata_name) {
                                *metadata_value = met_value.clone();
                                debug!("Updating metadata: {} with value: {:?}", metadata_name, met_value);
                            }
                        }
                    }
                }
                else => break,
            }
        }
        debug!("Finished processing user performances");
        user_performances
    })
}

/// Get a user performance or create a new one if it doesn't exist
fn get_performance_or_default<'a>(
    user_performances: &'a mut HashMap<String, ChatPerformance>,
    user_id: &'a str,
    metrics: &'a HashMap<String, f32>,
    metadatas: &'a HashMap<String, MetadataTypes>,
) -> &'a mut ChatPerformance {
    user_performances
        .entry(user_id.to_owned())
        .or_insert(ChatPerformance {
            id: user_id.to_owned(),
            perf_type: PerformanceType::Unknown,
            username: "".to_string(),
            avatar: "".to_string(),
            metrics: metrics.clone(),
            metadata: metadatas.clone(),
        })
}
