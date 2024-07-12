use core::panic;
use std::{fs, collections::HashMap};
use std::time::Instant;
use log::{debug, info};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::twitch_utils::TwitchAPIWrapper;
use crate::_types::twitchtypes::{ChatLog, Comment};
use crate::_types::clptypes::{UserChatPerformance, MetadataTypes};

use crate::metadata::get_metadata;
use crate::metrics::get_metrics;
use crate::leaderboards::get_leaderboards;

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

// impl<'a> ChatLogProcessor<'a> {
//     pub fn new(twitch: &'a TwitchAPIWrapper) -> Self {
//         Self { twitch }
//     }

//     pub fn __parse_to_log_struct(&self, chat_log_path: String) -> ChatLog {
//         let chat_log_str = fs::read_to_string(chat_log_path).unwrap();
//         let chat_log: ChatLog = serde_json::from_str(&chat_log_str).unwrap();
//         chat_log
//     }

//     pub async fn parse_from_log_object(&self, chat_log: ChatLog) -> Vec<UserChatPerformance> {
//         let start_time = Instant::now();
        
//         info!("Parsing chat log object");
//         let mut pre_performance: HashMap<String, UserChatPerformance> = HashMap::new();

//         let (metric_join_handles, metric_sender, mut metric_receiver) = get_metrics().await;
//         let (metadata_join_handles, metadata_sender, mut metadata_receiver) = get_metadata(self.twitch).await;

//         for (sequence_no, comment) in chat_log.comments.iter().enumerate() {
//             debug!("Processing comment by user: {} (message: {} of {})", comment.commenter._id, sequence_no, chat_log.comments.len());

//             let sequence_no = sequence_no as u32;
//             pre_performance.entry(comment.commenter._id.clone()).or_insert(UserChatPerformance {
//                 id: comment.commenter._id.clone(),
//                 username: comment.commenter.display_name.clone(),
//                 avatar: comment.commenter.logo.clone(),
//                 metrics: metric_structs.iter().map(|m| (m.get_name().to_string(), 0.0)).collect(),
//                 metadata: metadata_structs.iter().map(|m| (m.get_name().to_string(), m.get_default_value())).collect(),
//             });

//             let metric_update_map = metric_structs.iter_mut().map(|m| {
//                 let metric_name = m.get_name();
//                 let metric_score = m.get_metric(comment.clone(), sequence_no).clone();
//                 (metric_name.to_string(), metric_score)
//             });

//             let metadata_update_map = metadata_structs.iter().map(|m| {
//                 let metadata_name = m.get_name();
//                 let metadata_score = m.get_metadata(comment.clone(), sequence_no);
//                 (metadata_name.to_string(), metadata_score)
//             });

//             // debug!("Metric update map: {:?}", metric_update_map);
//             // debug!("Metadata update map: {:?}", metadata_update_map);

//             for (metric_name, update) in metric_update_map {
//                 for (user_id, met_value) in update.iter() {
//                     // NOTE: the user_id will definitely exist
//                     if let Some(user_chat_performance) = pre_performance.get_mut(user_id) {
//                         if let Some(metric_value) = user_chat_performance.metrics.get_mut(&metric_name) {
//                             *metric_value += met_value;
//                         }
//                     }
//                 }
//             }

//             for (metadata_name, update) in metadata_update_map {
//                 for (user_id, met_value) in update.iter() {
//                     // NOTE: the user_id will definitely exist
//                     if let Some(user_chat_performance) = pre_performance.get_mut(user_id) {
//                         if let Some(metadata_value) = user_chat_performance.metadata.get_mut(&metadata_name) {
//                             debug!("Updating metadata: {} with value: {:?}", metadata_name, met_value);
//                             *metadata_value = met_value.clone();
//                         }
//                     }
//                 }
//             }
//             debug!("User performance: {:?}", pre_performance.get(&comment.commenter._id).unwrap());
//         }

//         // Flush final metric updates
//         let metric_update_map = metric_structs.iter().map(|m| {
//             let metric_name = m.get_name();
//             let metric_score = m.finish();
//             (metric_name.to_string(), metric_score)
//         });

//         // debug!("Final metric update map: {:?}", metric_update_map);

//         for (metric_name, update) in metric_update_map {
//             for (user_id, met_value) in update.iter() {
//                 // NOTE: the user_id will definitely exist
//                 if let Some(user_chat_performance) = pre_performance.get_mut(user_id) {
//                     if let Some(metric_value) = user_chat_performance.metrics.get_mut(&metric_name) {
//                         *metric_value += met_value;
//                     }
//                 }
//             }
//         }

//         let elapsed = start_time.elapsed();
//         info!("Chat log processing took: {}ms", elapsed.as_millis());

//         pre_performance.values().cloned().collect()
//     }

//     #[allow(dead_code)]
//     async fn parse(&self, chat_log_path: String) -> Vec<UserChatPerformance> {
//         let chat_log = self.__parse_to_log_struct(chat_log_path);
//         self.parse_from_log_object(chat_log).await
//     }

//     pub fn export_to_leaderboards(performances: Vec<UserChatPerformance>) {
//         let mut leaderboards = get_leaderboards();
//         for leaderboard in leaderboards.iter_mut() {
//             for performance in performances.iter() {
//                 leaderboard.update_leaderboard(performance.clone());
//             }
//             leaderboard.save();
//         }
//     }
// }

impl<'a> ChatLogProcessor<'a> {
    pub fn new(twitch: &'a TwitchAPIWrapper) -> Self {
        Self { twitch }
    }

    pub fn __parse_to_log_struct(&self, chat_log_path: String) -> ChatLog {
        let chat_log_str = fs::read_to_string(chat_log_path).unwrap();
        let chat_log: ChatLog = serde_json::from_str(&chat_log_str).unwrap();
        chat_log
    }

    pub async fn parse_from_log_object(&self, chat_log: ChatLog) -> Vec<UserChatPerformance> {
        let start_time = Instant::now();
        let (metrics, _metric_join_handles, metric_sender, metric_receiver) = get_metrics().await;
        let (metadatas, _metadata_join_handles, metadata_sender, metadata_receiver) = get_metadata(self.twitch).await;

        info!("Parsing chat log object");
        spawn_chatlog_to_receiver(chat_log.clone(), vec![metric_sender, metadata_sender]);

        let performances = if let Ok(performances) = spawn_user_chat_perforance_thread(metrics, metric_receiver, metadatas, metadata_receiver).await {
            performances.to_owned()
        } else {
            panic!("Could not get user performances");
        };
        info!("Chat log processing took: {}ms", start_time.elapsed().as_millis());
        performances.values().cloned().collect()
    }

    #[allow(dead_code)]
    async fn parse(&self, chat_log_path: String) -> Vec<UserChatPerformance> {
        let chat_log = self.__parse_to_log_struct(chat_log_path);
        self.parse_from_log_object(chat_log).await
    }

    /// A function to export the user performances to the leaderboards and save them
    pub fn export_to_leaderboards(performances: Vec<UserChatPerformance>) {
        let mut leaderboards = get_leaderboards();
        for leaderboard in leaderboards.iter_mut() {
            for performance in performances.iter() {
                leaderboard.update_leaderboard(performance.clone());
            }
            leaderboard.save();
        }
    }
}

/// A function to apwn a thread to take a ChatLog and add its comments to a receiver
pub fn spawn_chatlog_to_receiver(chat_log: ChatLog, senders: Vec<broadcast::Sender<(Comment, u32)>>) -> JoinHandle<()> {
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
pub fn spawn_user_chat_perforance_thread(metrics: HashMap<String, f32>, mut metric_receiver: mpsc::Receiver<(String, HashMap<String, f32>)>, metadatas: HashMap<String, MetadataTypes>, mut metadata_receiver: mpsc::Receiver<(String, HashMap<String, MetadataTypes>)>) -> JoinHandle<HashMap<String, UserChatPerformance>> {
    tokio::task::spawn(async move {
        let mut user_performances: HashMap<String, UserChatPerformance> = HashMap::new();
        loop {
            tokio::select! {
                Some((metric_name, metric_update)) = metric_receiver.recv() => {
                    for (user_id, met_value) in metric_update.iter() {
                        get_performance_or_default(&mut user_performances, user_id, &metrics, &metadatas);
                        if let Some(user_chat_performance) = user_performances.get_mut(user_id) {
                            if let Some(metric_value) = user_chat_performance.metrics.get_mut(&metric_name) {
                                *metric_value += met_value;
                                debug!("Updating metric: {} with value: {:?}", metric_name, met_value);
                            }
                        }
                    }
                }
                Some((metadata_name, metadata_update)) = metadata_receiver.recv() => {
                    for (user_id, met_value) in metadata_update.iter() {
                        get_performance_or_default(&mut user_performances, user_id, &metrics, &metadatas);
                        if let Some(user_chat_performance) = user_performances.get_mut(user_id) {
                            if let Some(metadata_value) = user_chat_performance.metadata.get_mut(&metadata_name) {
                                *metadata_value = met_value.clone();
                                debug!("Updating metadata: {} with value: {:?}", metadata_name, met_value);
                            }
                        }
                    }
                }
                else => break,
            }
        };
        debug!("Finished processing user performances");
        user_performances
    })
}

/// Get a user performance or create a new one if it doesn't exist
fn get_performance_or_default<'a>(user_performances: &'a mut HashMap<String, UserChatPerformance>, user_id: &'a String, metrics: &'a HashMap<String, f32>, metadatas: &'a HashMap<String, MetadataTypes>) -> &'a mut UserChatPerformance {
    user_performances.entry(user_id.clone()).or_insert(UserChatPerformance {
        id: user_id.clone(),
        username: "".to_string(),
        avatar: "".to_string(),
        metrics: metrics.clone(),
        metadata: metadatas.clone(),
    })
}