use std::sync::mpsc;
use std::thread;

use rumqttc::{self, Client, Connection, Event, Incoming, MqttOptions, QoS};

#[derive(Clone, Debug)]
pub enum MqttLog {
    Info(String),
    Recv { topic: String, payload: String },
    Send { topic: String, payload: String },
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransportType {
    Tcp,
    Ws,
}

pub struct MqttManager {
    log_rx: mpsc::Receiver<MqttLog>,
    cmd_tx: Option<mpsc::Sender<MqttCmd>>,
    logs: Vec<MqttLog>,
    connected: bool,
}

enum MqttCmd {
    Publish { topic: String, qos: QoS, payload: String },
    Subscribe { topic: String },
    Disconnect,
}

impl MqttManager {
    pub fn new() -> Self {
        Self {
            log_rx: mpsc::channel().1,
            cmd_tx: None,
            logs: Vec::new(),
            connected: false,
        }
    }

    pub fn connected(&self) -> bool {
        self.connected
    }

    pub fn logs(&self) -> &[MqttLog] {
        &self.logs
    }

    /// Drain new messages from the channel into the internal log buffer
    pub fn poll(&mut self) {
        while let Ok(msg) = self.log_rx.try_recv() {
            match &msg {
                MqttLog::Info(s) if s == "Disconnected" => self.connected = false,
                _ => {}
            }
            self.logs.push(msg);
        }
    }

    pub fn connect(
        &mut self,
        host: &str,
        port: u16,
        path: &str,
        username: &str,
        password: &str,
        transport: TransportType,
    ) {
        if self.connected {
            return;
        }

        let client_id = format!("rust_test_{}", rand_hex());

        let mut opts = MqttOptions::new(&client_id, host, port);
        opts.set_credentials(username, password);
        opts.set_clean_session(true);
        opts.set_keep_alive(std::time::Duration::from_secs(30));

        match transport {
            TransportType::Tcp => {}
            TransportType::Ws => {
                let transport = rumqttc::Transport::Ws;
                opts.set_transport(transport);
            }
        }

        let (client, conn) = Client::new(opts, 10);
        let (log_tx, log_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel();

        self.log_rx = log_rx;
        self.cmd_tx = Some(cmd_tx);
        self.connected = true;

        let lt = log_tx.clone();
        lt.send(MqttLog::Info(format!("Connecting to {}:{}/{} ...", host, port, path)))
            .ok();

        thread::spawn(move || {
            run_eventloop(client, conn, cmd_rx, lt);
        });
    }

    pub fn publish(&self, topic: &str, qos: QoS, payload: &str) {
        if let Some(tx) = &self.cmd_tx {
            tx.send(MqttCmd::Publish {
                topic: topic.to_string(),
                qos,
                payload: payload.to_string(),
            })
            .ok();
        }
    }

    pub fn subscribe(&self, topic: &str) {
        if let Some(tx) = &self.cmd_tx {
            tx.send(MqttCmd::Subscribe {
                topic: topic.to_string(),
            })
            .ok();
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(tx) = &self.cmd_tx {
            tx.send(MqttCmd::Disconnect).ok();
        }
        self.cmd_tx = None;
        self.connected = false;
    }
}

fn run_eventloop(
    client: Client,
    mut connection: Connection,
    cmd_rx: mpsc::Receiver<MqttCmd>,
    log_tx: mpsc::Sender<MqttLog>,
) {
    log_tx
        .send(MqttLog::Info("Connection started, waiting for acknowledgement...".into()))
        .ok();

    for event in connection.iter() {
        // Check for commands
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                MqttCmd::Publish { topic, qos, payload } => {
                    if let Err(e) = client.publish(&topic, qos, false, payload.as_bytes()) {
                        log_tx
                            .send(MqttLog::Error(format!("Publish failed: {}", e)))
                            .ok();
                    } else {
                        log_tx
                            .send(MqttLog::Send {
                                topic: topic.clone(),
                                payload: payload.clone(),
                            })
                            .ok();
                    }
                }
                MqttCmd::Subscribe { topic } => {
                    if let Err(e) = client.subscribe(&topic, QoS::AtMostOnce) {
                        log_tx
                            .send(MqttLog::Error(format!("Subscribe failed: {}", e)))
                            .ok();
                    } else {
                        log_tx
                            .send(MqttLog::Info(format!("Subscribed: {}", topic)))
                            .ok();
                    }
                }
                MqttCmd::Disconnect => {
                    client.disconnect().ok();
                    log_tx.send(MqttLog::Info("Disconnected".into())).ok();
                    return;
                }
            }
        }

        match event {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                log_tx
                    .send(MqttLog::Info("Connected!".into()))
                    .ok();
            }
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                let payload = String::from_utf8_lossy(&p.payload).to_string();
                log_tx
                    .send(MqttLog::Recv {
                        topic: p.topic,
                        payload,
                    })
                    .ok();
            }
            Ok(Event::Incoming(pkt)) => {
                log_tx
                    .send(MqttLog::Info(format!("{:?}", pkt)))
                    .ok();
            }
            Ok(Event::Outgoing(_)) => {}
            Err(e) => {
                log_tx
                    .send(MqttLog::Error(format!("Connection error: {}", e)))
                    .ok();
                break;
            }
        }
    }

    log_tx.send(MqttLog::Info("Disconnected".into())).ok();
}

fn rand_hex() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}", t)
}
