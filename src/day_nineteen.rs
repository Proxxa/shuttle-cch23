use std::{collections::HashMap, sync::{RwLock, Arc, atomic::AtomicUsize}};

use rocket::{*, futures::{SinkExt, StreamExt}, serde::json::Json};
use rocket_ws as ws;
use ::serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{channel, Sender, Receiver};

#[get("/ws/ping")]
pub fn ping_pong(sock: ws::WebSocket) -> ws::Stream!['static] {
    let mut playing_game = false;
    ws::Stream! { sock =>
        for await msg in sock {
            let msg = msg?;
            match msg {
                ws::Message::Text(s) => 
                    match s.as_str() {
                        "serve" => {
                            playing_game = true;
                            continue
                        },
                        "ping" => if playing_game { yield ws::Message::text("pong")}
                        _ => continue,
                    },
                _ => yield ws::Message::Close(None),
            }
        }
    }
}


#[derive(Default)]
pub struct RoomSenderHolder(pub RwLock<HashMap<i32, Arc<Room>>>);

pub struct Room(Sender<SignedTweet>);

impl Room {
    fn connect(&self) -> Receiver<SignedTweet> {
        self.0.subscribe()
    }

    async fn send(&self, tweet: SignedTweet) -> Result<usize, tokio::sync::broadcast::error::SendError<SignedTweet>> {
        self.0.send(tweet)
    }
}


#[derive(Default)]
pub struct TweetViewCounter(Arc<AtomicUsize>);

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct UnsignedTweet {
    message: String,
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SignedTweet {
    user: String,
    message: String,
}

#[post("/reset")]
pub fn reset_views(tvc: &State<TweetViewCounter>) {
    tvc.0.store(0, std::sync::atomic::Ordering::Release);
}

#[get("/views")]
pub fn get_views(tvc: &State<TweetViewCounter>) -> Json<usize> {
    Json(tvc.0.load(std::sync::atomic::Ordering::Acquire))
}

#[get("/ws/room/<room>/user/<user>")]
pub fn twitter_sock(sock: ws::WebSocket, room: i32, user: String, rsh: &State<RoomSenderHolder>, tvc: &State<TweetViewCounter>) -> ws::Channel<'static> {
    let tvc = tvc.0.clone();
    let mut rcv = rsh.0.write().unwrap().entry(room).or_insert_with(|| Arc::new(Room(channel(1024).0))).connect();
    let room = rsh.0.read().unwrap().get(&room).unwrap().clone();
    use ws::Message::*;
    sock.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                tokio::select! {
                    message = stream.next() => {
                        let Some(message) = message else { return Ok(()) };
                            match message? {
                                Text(ref message) => {
                                    let Ok(UnsignedTweet { message }) = serde_json::from_str(message) else { continue; };
                                    if message.len() > 128 { continue; }
                                    let _ = room.send(SignedTweet {
                                        user: user.clone(),
                                        message,
                                    }).await;
                                },
                                Ping(d) => stream.send(Pong(d)).await?,
                                Close(_) => {
                                    break;
                                },
                                _ => (),
                            }
                        }
                        tweet = rcv.recv() => {
                            let message = serde_json::to_string(&tweet.unwrap()).unwrap();
                            stream.send(Text(message.clone())).await?;
                            tvc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            info!("Sent {message}");
                        }
                    }
                }
            Ok(()) 
        })
    })
}