#[macro_use] extern crate rocket;

use rocket::{State, Shutdown};
use rocket::fs::{relative, FileServer};
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::response::stream::{EventStream, Event};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;
use std::thread;

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use nm::Message;

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct ServerInfo {
    #[field(validate = len(..30))]
    pub version: String,
    #[field(validate = len(..30))]
    pub name: String,
}

#[get("/info")]
fn get_info() -> Json<ServerInfo> {
    let info = ServerInfo {
        version: String::from("1.0.0"), 
        name: String::from("TestSite")
    };

    Json(info)
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}
pub struct CacheControl;

#[rocket::async_trait]
impl Fairing for CacheControl {
    fn info(&self) -> Info {
        Info {
            name: "Attaching cache header to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Cache-Control", "no-transform"));
    }
}

#[launch]
fn rocket() -> _ {
    let sender = channel::<Message>(1024).0;
    let sender2 = Sender::clone(&sender);

    thread::spawn(move || {
        let _result = {
            nm::run(sender2)
        };
    }); 

    rocket::build()
        .manage(sender)
        .mount("/api", routes![events])
        .mount("/api", routes![get_info])
        .mount("/", FileServer::from(relative!("static")))
        .attach(CacheControl{})
        
}
