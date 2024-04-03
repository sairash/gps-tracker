
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json;
use socketioxide::{adapter::LocalAdapter, Socket};

#[derive(Deserialize, Clone, Debug)]
struct UserId(String);

#[derive(Deserialize, Clone, Debug, Copy)]
struct Lat(f64);
impl Lat {
    fn get_value(self) -> f64 {
        self.0
    }
}

#[derive(Deserialize, Clone, Debug, Copy)]
struct Lng(f64);
impl Lng {
    fn get_value(self) -> f64 {
        self.0
    }
}

#[derive(Deserialize, Clone, Debug)]
struct Room(String);

#[derive(Deserialize, Clone, Debug)]
struct LastUpdate(String);
impl LastUpdate {
    fn get_value(self) -> String {
        self.0
    }
}

#[derive(Deserialize)]
struct Auth {
    pub user_id: UserId,
    pub lat: Lat,
    pub lng: Lng,
    pub last_update: LastUpdate,
    pub room: Room,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    lat: f64,
    lng: f64,
    last_update: String
}

#[derive(Serialize, Deserialize, Debug)]
struct LocationData {
    id: String,
    location: Location,
}

pub async fn handler(socket: Arc<Socket<LocalAdapter>>) {
    // info!("Socket connected on / with id: {}", socket.sid);
    match socket.handshake.data::<Auth>() {
        Ok(c) => {
            socket.extensions.insert(c.user_id.clone());
            socket.extensions.insert(c.room.clone());
            socket.extensions.insert(c.lat);
            socket.extensions.insert(c.lng);
            socket.extensions.insert(c.last_update.clone());


            let locations_to_send = Arc::new(Mutex::new(Vec::new()));

            socket.within(c.room.clone().0).sockets().unwrap().into_iter().enumerate().for_each(|(_n, s)| {
            
                let mut location = Location { lat: 0.0, lng: 0.0 , last_update: String::from("")};
                if let Some(ref lng_) = s.extensions.get::<Lng>() {
                    location.lng = lng_.get_value()
                }
                if let Some(ref lat_) = s.extensions.get::<Lat>() {
                    location.lat = lat_.get_value();
                }

                if let Some(ref last_up) = s.extensions.get::<LastUpdate>() {
                    location.last_update = last_up.value().0.clone();
                }

                let mut rider_location = LocationData {
                    id: "0".to_string(),
                    location: location,
                };
                if let Some(ref rider_id_) = s.extensions.get::<UserId>() {
                    rider_location.id = rider_id_.0.to_string();
                }
                locations_to_send.lock().unwrap().push(rider_location);
            });

            let json_string = serde_json::to_string(&*locations_to_send).unwrap();
            socket.emit("location_update", json_string).ok();
            socket.emit("info", "Welcome to the ride!").ok();

            let _ = socket.to(c.room.clone().0).emit("location_update", serde_json::to_string(&LocationData{
                id: c.user_id.0,
                location: Location{
                    lat: c.lat.get_value(), 
                    lng: c.lng.get_value(),
                    last_update: c.last_update.0,
                }
            }).unwrap());
            socket.join(c.room.0).unwrap();
        }
        Err(c) => {
            println!("{}", c);
            socket.disconnect().ok();
            return;
        }
    }

    socket.on(
    "location_update",
    |socket, loc: Location, _, _| async move {
        socket.extensions.insert(loc.lat);
        socket.extensions.insert(loc.lng);
        socket.extensions.insert(loc.last_update.clone());

        let UserId(ref user_id) = *socket.extensions.get().unwrap();
        let Room(ref room) = *socket.extensions.get().unwrap();

        socket
            .to(String::from(room))
            .emit(
                "location_update",
                serde_json::to_string(&LocationData {
                    id: user_id.to_string(),
                    location: loc,
                })
                .unwrap(),
            )
            .unwrap();
        },
    );
}
