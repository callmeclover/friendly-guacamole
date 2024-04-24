mod message;
mod user;
use message::{ model::*, func::{ into_censored_md, VecWithHardLimit } };
use message::;
use user::{ model::*, auth::* };

use chrono::Utc;
use axum::{
    extract::{ connect_info::ConnectInfo, State, ws::{ Message, WebSocket, WebSocketUpgrade } },
    response::IntoResponse,
    routing::get,
    Router,
};
use pulldown_cmark::{ Parser, html::push_html };
use ammonia::clean;
use std::{ net::SocketAddr, path::PathBuf, collections::HashSet, sync::{ Arc, Mutex } };
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use tower_http::{ services::ServeDir, trace::{ DefaultMakeSpan, TraceLayer } };
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt };
use futures::{ sink::SinkExt, stream::StreamExt };

static MESSAGES: Lazy<Mutex<Vec<MessageSent>>> = Lazy::new(|| Mutex::new(Vec::with_capacity(20)));
static USER_ID: Lazy<Arc<Mutex<i32>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

lazy_static::lazy_static! {
    static ref DB_CLIENT: Arc<Mutex<DatabaseConnectix>> = {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            DatabaseConnectix::default();
        })
    };
}

// Our shared state
struct AppState {
    // We require unique usernames. This tracks which usernames have been taken.
    user_set: Mutex<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber
        ::registry()
        .with(
            tracing_subscriber::EnvFilter
                ::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // Set up application state for use with with_state().
    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState { user_set, tx });

    // build our application with some routes
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http().make_span_with(
                DefaultMakeSpan::default().include_headers(true)
            )
        );

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:9067").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

/// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, _who: SocketAddr, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    *USER_ID.lock().unwrap() += 1;
    let user_id = USER_ID.lock().unwrap().clone();

    let user = Arc::new(Mutex::new(User::new("".into(), user_id)));

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.tx.subscribe();

    // Now send the "joined" message to all subscribers.
    let msg = format!("user with id {0} connected.", user.lock().unwrap().id);
    tracing::debug!("{msg}");

    let msg_vec = (*MESSAGES.lock().unwrap().clone()).to_vec();
    let _ = sender.send(
        Message::Text(
            serde_json
                ::to_string(&(RetrieveMessages { msgs: msg_vec }))
                .expect("couldn't serialize MESSAGES vector!")
        )
    ).await;

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state.tx.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let user_recv = user.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let message = serde_json
                ::from_str::<MessageTypes>(&text)
                .expect("couldn't get json from message");
            match message {
                MessageTypes::MessageSent(mut request) => {
                    let mut msg_new: String = String::new();
                    push_html(
                        &mut msg_new,
                        Parser::new(&request.msg.replace('<', "&lt;").replace('>', "&gt;"))
                    );

                    match into_censored_md(&clean(&msg_new), &mut user_recv.lock().unwrap()) {
                        Ok(output) => {
                            request.msg = output;
                            request.time = Some(Utc::now());
                            let mut msg_vec = MESSAGES.lock().unwrap();
                            msg_vec.push_with_hard_limit(&request);
                            let _ = tx.send(
                                serde_json
                                    ::to_string(&request)
                                    .expect("couldnt convert json to string")
                            );
                            continue;
                        }
                        Err(reason) => {
                            println!(
                                "Message blocked from user '{}' for reason '{}'",
                                request.user,
                                reason
                            );
                            continue;
                        }
                    }
                },
                MessageTypes::UserJoin(request) => {
                    user_recv.lock().unwrap().name = request.user;
                    let _ = state.tx.send(
                        serde_json::to_string(&(UserJoin { userjoin: user_recv.lock().unwrap().name.clone() })).expect("")
                    );
                    continue;
                }
                _ => {
                    continue;
                }
            }
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{0} left.", user.lock().unwrap().name);
    tracing::debug!("{msg}");
    let _ = state.tx.send(
        serde_json::to_string(&(UserLeft { userleft: user.lock().unwrap().name.clone() })).expect("")
    );

    *USER_ID.lock().unwrap() -= 1;
}
