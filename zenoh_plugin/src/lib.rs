use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use cdr::{CdrLe, Infinite};
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;
use turtle_core::events::MoveEvent;
use zenoh::config::{Config, EndPoint};
use zenoh::prelude::*;
use zenoh::Session;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Twist {
    linear: Vector3,
    angular: Vector3,
}

#[derive(Deserialize, PartialEq)]
struct Time {
    sec: i32,
    nanosec: u32,
}

#[derive(Deserialize, PartialEq)]
struct Log {
    stamp: Time,
    level: u8,
    name: String,
    msg: String,
    file: String,
    function: String,
    line: u32,
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}.{}] [{}]: {}",
            self.stamp.sec, self.stamp.nanosec, self.name, self.msg
        )
    }
}

async fn pub_twist(session: &Session, cmd_key: &KeyExpr<'_>, linear: i32, angular: i32) {
    let twist = Twist {
        linear: Vector3 {
            x: linear as f64,
            y: 0.0,
            z: 0.0,
        },
        angular: Vector3 {
            x: 0.0,
            y: 0.0,
            z: angular as f64,
        },
    };

    let encoded = cdr::serialize::<_, _, CdrLe>(&twist, Infinite).unwrap();
    if let Err(e) = session.put(cmd_key, encoded).await {
        warn!("Error writing to zenoh: {}", e);
    }
}

#[derive(Debug)]
struct TwistEvent(pub Twist);

pub struct ZenohPlugin;

impl Plugin for ZenohPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(start_zenoh_session)
            .add_system(handle_input_event)
            .add_system(handle_teleop_event)
            .add_event::<TwistEvent>();
    }
}

fn start_zenoh_session(mut commands: Commands, task_pool: Res<IoTaskPool>) {
    let (sender, receiver) = mpsc::unbounded_channel::<MoveEvent>();
    let (twist_sender, twist_receiver) = mpsc::unbounded_channel::<Twist>();

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    task_pool.spawn(zehno_message_loop(receiver, twist_sender)).detach();
    info!("insert_resource sender close?{}", sender.is_closed());
    commands.insert_resource(sender);
    commands.insert_resource(twist_receiver);
}

fn handle_input_event(
    mut move_event_reader: EventReader<MoveEvent>,
    sender: Res<UnboundedSender<MoveEvent>>,

) {
    if sender.is_closed() {
        warn!("closed");
        return;
    }
    //let sender = sender.unwrap();
    for event in move_event_reader.iter() {
        if !event.teleop {
            sender.send(*event).unwrap();
        }
    }
}

fn handle_teleop_event(mut move_event_writer: EventWriter<MoveEvent>,
                       mut twist_receiver: ResMut<UnboundedReceiver<Twist>>,){
    if let Ok(twist) = twist_receiver.try_recv() {
        move_event_writer.send(MoveEvent{
            rotation: twist.angular.z as i32,
            movement: twist.linear.x as i32,
            teleop: true,
        });
    }
}

async fn zehno_message_loop(
    mut move_event_receiver: UnboundedReceiver<MoveEvent>,
    //twist_sender: UnboundedSender<Twist>,
    twist_sender: UnboundedSender<Twist>
) {
    let mut config = Config::default();
    config
        .connect
        .endpoints
        .push(EndPoint::from_str("tcp/127.0.0.1:7447").unwrap());
    let rosout = "/rt/rosout".to_string();
    let cmd_vel = "/rt/turtle1/cmd_vel".to_string();
    // ResKey for publication on "cmd_vel" topic
    let cmd_key = KeyExpr::from(cmd_vel);

    info!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    info!("Subscriber on {}", rosout);

    let mut subscriber = session.subscribe(&rosout).await.unwrap();
    let mut cmd_vel_sub = session.subscribe(&cmd_key).await.unwrap();

    loop {
        tokio::select!(
            // On sample received by the subscriber
            sample = subscriber.next()=> {
                let sample: Sample = sample.unwrap();
                // copy to be removed if possible
                // let buf = sample.payload.to_vec();
                match cdr::deserialize::<Log>(&sample.value.payload.contiguous())  {
                    Ok(log) => {
                        info!("{}", log);
                        //std::io::stdout().execute(MoveToColumn(0)).unwrap();
                    }
                    Err(e) => warn!("Error decoding Log: {}", e),
                }
            },
            sample = cmd_vel_sub.next() => {
                let sample: Sample = sample.unwrap();
                match cdr::deserialize::<Twist>(&sample.value.payload.contiguous())  {
                    Ok(cmd_vel) => {
                        twist_sender.send(cmd_vel).unwrap();
                        //std::io::stdout().execute(MoveToColumn(0)).unwrap();
                    }
                    Err(e) => warn!("Error decoding Log: {}", e),
                }
            },
            event = move_event_receiver.recv() => {
                if let Some(motion) = event {
                    pub_twist(&session, &cmd_key, motion.movement, motion.rotation).await;
                }
                else {
                    warn!("no more event, exit");
                    break;
                }
            },
        );
    }
}
