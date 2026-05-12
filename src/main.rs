#![windows_subsystem = "windows"]

mod mqtt_client;

use eframe::egui::{self, Color32, RichText};
use mqtt_client::{MqttLog, MqttManager, TransportType};
use rumqttc::QoS;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_title("MQTT Test"),
        ..Default::default()
    };

    eframe::run_native(
        "MQTT Test",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            Ok(Box::new(MqttApp::default()))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_data = std::fs::read("resources/fonts/MiSans-Normal.ttf").unwrap_or_else(|_| {
        // fallback: try relative to exe
        let mut exe = std::env::current_exe().unwrap_or_default();
        exe.pop();
        exe.push("resources/fonts/MiSans-Normal.ttf");
        std::fs::read(&exe).unwrap_or_default()
    });
    if !font_data.is_empty() {
        fonts
            .font_data
            .insert("MiSans".to_string(), egui::FontData::from_owned(font_data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "MiSans".to_string());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("MiSans".to_string());
        ctx.set_fonts(fonts);
    }
}

fn setup_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.widgets.noninteractive.bg_fill = Color32::from_gray(30);
    visuals.widgets.inactive.bg_fill = Color32::from_gray(40);
    visuals.widgets.hovered.bg_fill = Color32::from_gray(50);
    visuals.widgets.active.bg_fill = Color32::from_gray(60);
    ctx.set_visuals(visuals);
}

struct MqttApp {
    host: String,
    port: String,
    path: String,
    username: String,
    password: String,
    transport: TransportType,

    sub_topic: String,
    subscriptions: Vec<String>,

    pub_topic: String,
    pub_msg: String,
    pub_qos: usize,

    manager: MqttManager,
}

impl Default for MqttApp {
    fn default() -> Self {
        Self {
            host: "47.109.52.219".to_string(),
            port: "1883".to_string(),
            path: "/mqtt".to_string(),
            username: "longzheng0315".to_string(),
            password: String::new(),
            transport: TransportType::Tcp,

            sub_topic: String::new(),
            subscriptions: Vec::new(),

            pub_topic: String::new(),
            pub_msg: String::new(),
            pub_qos: 0,

            manager: MqttManager::new(),
        }
    }
}

impl eframe::App for MqttApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.manager.poll();
        ui.ctx().request_repaint();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("MQTT Test");
            ui.add_space(8.0);

            self.ui_connection(ui);
            ui.add_space(8.0);
            self.ui_subscribe(ui);
            ui.add_space(8.0);
            self.ui_publish(ui);
            ui.add_space(8.0);
            self.ui_log(ui);
            ui.add_space(8.0);
            self.ui_statusbar(ui);
        });
    }
}

impl MqttApp {
    fn ui_connection(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Connection")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Transport:");
                    ui.radio_value(&mut self.transport, TransportType::Tcp, "TCP (1883)");
                    ui.radio_value(&mut self.transport, TransportType::Ws, "WebSocket (9001)");
                });

                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.host);
                });

                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.port);

                    if self.transport == TransportType::Tcp {
                        if self.port == "9001" || self.port.is_empty() {
                            self.port = "1883".into();
                        }
                    } else if self.port == "1883" || self.port.is_empty() {
                        self.port = "9001".into();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut self.path);
                });

                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.username);
                });

                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                });

                let btn_text = if self.manager.connected() {
                    "Disconnect"
                } else {
                    "Connect"
                };

                let btn_color = if self.manager.connected() {
                    Color32::from_rgb(231, 76, 60)
                } else {
                    Color32::from_rgb(46, 204, 113)
                };

                if ui
                    .add(
                        egui::Button::new(RichText::new(btn_text).color(Color32::WHITE))
                            .fill(btn_color),
                    )
                    .clicked()
                {
                    if self.manager.connected() {
                        self.manager.disconnect();
                    } else {
                        let port: u16 = self.port.parse().unwrap_or(1883);
                        self.manager.connect(
                            &self.host,
                            port,
                            &self.path,
                            &self.username,
                            &self.password,
                            self.transport,
                        );
                    }
                }
            });
    }

    fn ui_subscribe(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Subscribe")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Topic:");
                    ui.text_edit_singleline(&mut self.sub_topic);
                    if ui.button("Subscribe").clicked() && !self.sub_topic.is_empty() {
                        self.manager.subscribe(&self.sub_topic);
                        if !self.subscriptions.contains(&self.sub_topic) {
                            self.subscriptions.push(self.sub_topic.clone());
                        }
                    }
                });

                if !self.subscriptions.is_empty() {
                    ui.label("Subscribed topics:");
                    for t in &self.subscriptions {
                        ui.label(format!("  - {}", t));
                    }
                }
            });
    }

    fn ui_publish(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Publish")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Topic:");
                    ui.text_edit_singleline(&mut self.pub_topic);
                });

                ui.horizontal(|ui| {
                    ui.label("QoS:");
                    egui::ComboBox::from_id_salt("qos")
                        .selected_text(format!("{}", self.pub_qos))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.pub_qos, 0, "0 - At most once");
                            ui.selectable_value(&mut self.pub_qos, 1, "1 - At least once");
                            ui.selectable_value(&mut self.pub_qos, 2, "2 - Exactly once");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Message:");
                    ui.text_edit_singleline(&mut self.pub_msg);
                });

                if ui.button("Publish").clicked() && !self.pub_topic.is_empty() {
                    let qos = match self.pub_qos {
                        1 => QoS::AtLeastOnce,
                        2 => QoS::ExactlyOnce,
                        _ => QoS::AtMostOnce,
                    };
                    self.manager.publish(&self.pub_topic, qos, &self.pub_msg);
                }
            });
    }

    fn ui_log(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Log")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for msg in self.manager.logs() {
                            match msg {
                                MqttLog::Info(s) => {
                                    ui.label(
                                        RichText::new(format!("[INFO] {}", s))
                                            .color(Color32::YELLOW),
                                    );
                                }
                                MqttLog::Recv { topic, payload } => {
                                    ui.label(
                                        RichText::new(format!("[RECV] [{}] {}", topic, payload))
                                            .color(Color32::from_rgb(46, 204, 113)),
                                    );
                                }
                                MqttLog::Send { topic, payload } => {
                                    ui.label(
                                        RichText::new(format!("[SEND] [{}] {}", topic, payload))
                                            .color(Color32::from_rgb(52, 152, 219)),
                                    );
                                }
                                MqttLog::Error(s) => {
                                    ui.label(
                                        RichText::new(format!("[ERR] {}", s))
                                            .color(Color32::from_rgb(231, 76, 60)),
                                    );
                                }
                            }
                        }
                    });
            });
    }

    fn ui_statusbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (color, text) = if self.manager.connected() {
                (Color32::from_rgb(46, 204, 113), "Connected")
            } else {
                (Color32::from_rgb(231, 76, 60), "Disconnected")
            };

            let painter = ui.painter();
            let pos = ui.cursor().min + egui::vec2(6.0, 8.0);
            painter.circle_filled(pos, 4.0, color);
            ui.add_space(16.0);
            ui.label(text);
        });
    }
}
