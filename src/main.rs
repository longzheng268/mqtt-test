#![windows_subsystem = "windows"]

mod mqtt_client;

use std::sync::Arc;

use eframe::egui::{self, Color32, RichText, Stroke};
use mqtt_client::{MqttLog, MqttManager, TransportType};
use rumqttc::QoS;
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 700.0])
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
    const FONT_BYTES: &[u8] = include_bytes!("../resources/fonts/MiSans-Normal.ttf");
    fonts.font_data.insert("MiSans".into(), Arc::new(egui::FontData::from_static(FONT_BYTES)));
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "MiSans".into());
    fonts.families.entry(egui::FontFamily::Monospace).or_default().push("MiSans".into());
    ctx.set_fonts(fonts);
}

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let vis = &mut style.visuals;
    vis.dark_mode = true;
    vis.window_fill = Color32::from_rgb(24, 26, 31);
    vis.panel_fill = Color32::from_rgb(24, 26, 31);
    vis.extreme_bg_color = Color32::from_rgb(18, 20, 24);
    vis.faint_bg_color = Color32::from_rgb(32, 34, 39);
    vis.window_shadow = egui::epaint::Shadow::NONE;

    let r = egui::CornerRadius::same(4);
    for w in [&mut vis.widgets.noninteractive, &mut vis.widgets.inactive, &mut vis.widgets.hovered, &mut vis.widgets.active, &mut vis.widgets.open] {
        w.corner_radius = r;
    }

    vis.widgets.noninteractive.bg_fill = Color32::from_rgb(36, 38, 44);
    vis.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(160, 165, 175));
    vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 58, 65));

    vis.widgets.inactive.bg_fill = Color32::from_rgb(42, 44, 50);
    vis.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(200, 205, 215));
    vis.widgets.inactive.bg_stroke = Stroke::new(1.5, Color32::from_rgb(65, 68, 78));

    vis.widgets.hovered.bg_fill = Color32::from_rgb(48, 50, 58);
    vis.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    vis.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::from_rgb(64, 128, 216));

    vis.widgets.active.bg_fill = Color32::from_rgb(55, 58, 66);
    vis.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    vis.widgets.active.bg_stroke = Stroke::new(1.5, Color32::from_rgb(64, 128, 216));

    vis.selection.bg_fill = Color32::from_rgb(40, 88, 168);
    vis.selection.stroke = Stroke::new(1.0, Color32::from_rgb(64, 128, 216));

    ctx.set_style(style);
}

// -- i18n --
#[derive(Clone, Copy, PartialEq)]
enum Lang { Zh, En }

struct Texts {
    connection: &'static str,
    transport: &'static str,
    host: &'static str,
    port: &'static str,
    path: &'static str,
    user: &'static str,
    pass: &'static str,
    tcp: &'static str,
    ws: &'static str,
    connect: &'static str,
    disconnect: &'static str,
    subscribe: &'static str,
    publish: &'static str,
    topic: &'static str,
    qos: &'static str,
    msg: &'static str,
    sub_btn: &'static str,
    pub_btn: &'static str,
    log: &'static str,
    clear: &'static str,
    save: &'static str,
    load: &'static str,
    connected: &'static str,
    disconnected: &'static str,
    subscribed: &'static str,
    info: &'static str,
    recv: &'static str,
    send: &'static str,
    err: &'static str,
    save_tip: &'static str,
    load_tip: &'static str,
}

const ZH: Texts = Texts {
    connection: "连接", transport: "传输方式", host: "主机", port: "端口", path: "路径",
    user: "用户名", pass: "密码", tcp: "TCP 1883", ws: "WebSocket 9001",
    connect: "连接", disconnect: "断开", subscribe: "订阅", publish: "发布",
    topic: "主题", qos: "服务质量", msg: "消息", sub_btn: "订阅", pub_btn: "发布",
    log: "日志", clear: "清空", save: "保存配置", load: "加载配置",
    connected: "已连接", disconnected: "未连接", subscribed: "已订阅主题",
    info: "信息", recv: "收到", send: "发送", err: "错误",
    save_tip: "将当前连接配置保存到 JSON 文件", load_tip: "从 JSON 文件加载连接配置",
};

const EN: Texts = Texts {
    connection: "Connection", transport: "Transport", host: "Host", port: "Port", path: "Path",
    user: "User", pass: "Pass", tcp: "TCP 1883", ws: "WebSocket 9001",
    connect: "Connect", disconnect: "Disconnect", subscribe: "Subscribe", publish: "Publish",
    topic: "Topic", qos: "QoS", msg: "Message", sub_btn: "Subscribe", pub_btn: "Publish",
    log: "Log", clear: "Clear", save: "Save", load: "Load",
    connected: "Connected", disconnected: "Disconnected", subscribed: "Subscribed Topics",
    info: "INFO", recv: "RECV", send: "SEND", err: "ERR",
    save_tip: "Save connection config to JSON", load_tip: "Load connection config from JSON",
};

// -- config --
#[derive(Serialize, Deserialize, Clone)]
struct MqttConfig {
    host: String,
    port: String,
    path: String,
    username: String,
    password: String,
    transport: String,
}

// -- app --
struct MqttApp {
    lang: Lang,
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
        let mut host = String::new();
        let mut port = "1883".to_string();
        let mut path = "/mqtt".to_string();
        let mut username = String::new();
        let mut password = String::new();
        let mut transport = TransportType::Tcp;

        if let Ok(data) = std::fs::read_to_string(config_file_path()) {
            if let Ok(c) = serde_json::from_str::<MqttConfig>(&data) {
                host = c.host;
                port = c.port;
                path = c.path;
                username = c.username;
                password = c.password;
                transport = if c.transport == "ws" { TransportType::Ws } else { TransportType::Tcp };
            }
        }

        Self {
            lang: Lang::Zh,
            host, port, path, username, password, transport,
            sub_topic: String::new(), subscriptions: Vec::new(),
            pub_topic: String::new(), pub_msg: String::new(), pub_qos: 0,
            manager: MqttManager::new(),
        }
    }
}

impl eframe::App for MqttApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.manager.poll();
        ui.ctx().request_repaint();
        let t = if self.lang == Lang::Zh { &ZH } else { &EN };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);

            // Title + language switch
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.label(RichText::new("MQTT 调试工具").size(18.0).strong().color(Color32::from_rgb(64, 140, 240)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(self.lang == Lang::En, "EN").clicked() { self.lang = Lang::En; }
                    if ui.selectable_label(self.lang == Lang::Zh, "中文").clicked() { self.lang = Lang::Zh; }
                });
            });
            ui.add_space(8.0);

            self.ui_connection(ui, t);
            ui.add_space(6.0);
            self.ui_pubsub(ui, t);
            ui.add_space(6.0);
            self.ui_log(ui, t);
            ui.add_space(4.0);
            self.ui_statusbar(ui, t);
            ui.add_space(4.0);
        });
    }
}

fn section_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(Color32::from_rgb(30, 32, 38))
        .stroke(Stroke::new(1.0, Color32::from_rgb(48, 52, 60)))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin { left: 12, right: 12, top: 8, bottom: 10 })
}

fn input_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(Color32::from_rgb(180, 185, 195)));
        ui.add_space(4.0);
        egui::Frame::NONE
            .fill(Color32::from_rgb(20, 22, 26))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 64, 72)))
            .corner_radius(egui::CornerRadius::same(3))
            .inner_margin(egui::Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.text_edit_singleline(value);
            });
    });
}

impl MqttApp {
    fn ui_connection(&mut self, ui: &mut egui::Ui, t: &Texts) {
        section_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(t.connection).size(14.0).strong().color(Color32::from_rgb(200, 210, 225)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(RichText::new(t.save).size(11.0)).on_hover_text(t.save_tip).clicked() { self.save_config(); }
                    if ui.small_button(RichText::new(t.load).size(11.0)).on_hover_text(t.load_tip).clicked() { self.load_config(); }
                });
            });
            ui.add_space(6.0);

            // Transport
            ui.horizontal(|ui| {
                ui.label(RichText::new(t.transport).size(13.0).color(Color32::from_rgb(180, 185, 195)));
                ui.add_space(8.0);
                ui.radio_value(&mut self.transport, TransportType::Tcp, t.tcp);
                ui.add_space(4.0);
                ui.radio_value(&mut self.transport, TransportType::Ws, t.ws);
            });
            ui.add_space(6.0);

            // Two columns of fields
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    input_field(ui, t.host, &mut self.host);
                    ui.add_space(3.0);
                    input_field(ui, t.port, &mut self.port);
                    ui.add_space(3.0);
                    input_field(ui, t.path, &mut self.path);
                });
                cols[1].vertical(|ui| {
                    input_field(ui, t.user, &mut self.username);
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(t.pass).size(13.0).color(Color32::from_rgb(180, 185, 195)));
                        ui.add_space(4.0);
                        egui::Frame::NONE
                            .fill(Color32::from_rgb(20, 22, 26))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 64, 72)))
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(6, 3))
                            .show(ui, |ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                            });
                    });
                });
            });

            // Auto port
            if self.transport == TransportType::Tcp && (self.port == "9001" || self.port.is_empty()) {
                self.port = "1883".into();
            } else if self.transport == TransportType::Ws && (self.port == "1883" || self.port.is_empty()) {
                self.port = "9001".into();
            }

            ui.add_space(8.0);

            // Connect button full width
            let (txt, clr) = if self.manager.connected() {
                (t.disconnect, Color32::from_rgb(200, 70, 70))
            } else {
                (t.connect, Color32::from_rgb(40, 160, 90))
            };
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new(RichText::new(txt).size(14.0).color(Color32::WHITE).strong())
                    .fill(clr)
                    .corner_radius(egui::CornerRadius::same(4))
                    .min_size(egui::vec2(ui.available_width() - 20.0, 30.0));
                if ui.add(btn).clicked() {
                    if self.manager.connected() {
                        self.manager.disconnect();
                    } else {
                        let port: u16 = self.port.parse().unwrap_or(1883);
                        self.manager.connect(&self.host, port, &self.path, &self.username, &self.password, self.transport);
                    }
                }
            });
        });
    }

    fn ui_pubsub(&mut self, ui: &mut egui::Ui, t: &Texts) {
        ui.columns(2, |cols| {
            // Subscribe
            section_frame().show(&mut cols[0], |ui| {
                ui.label(RichText::new(t.subscribe).size(14.0).strong().color(Color32::from_rgb(200, 210, 225)));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t.topic).size(13.0).color(Color32::from_rgb(180, 185, 195)));
                    ui.add_space(4.0);
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(20, 22, 26))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(60, 64, 72)))
                        .corner_radius(egui::CornerRadius::same(3))
                        .inner_margin(egui::Margin::symmetric(6, 3))
                        .show(ui, |ui| {
                            ui.text_edit_singleline(&mut self.sub_topic);
                        });
                    ui.add_space(2.0);
                    if ui.small_button(t.sub_btn).clicked() && !self.sub_topic.is_empty() {
                        self.manager.subscribe(&self.sub_topic);
                        if !self.subscriptions.contains(&self.sub_topic) {
                            self.subscriptions.push(self.sub_topic.clone());
                        }
                    }
                });
                if !self.subscriptions.is_empty() {
                    ui.add_space(4.0);
                    ui.label(RichText::new(t.subscribed).size(11.0).color(Color32::from_rgb(140, 150, 165)));
                    for s in &self.subscriptions {
                        ui.label(RichText::new(format!("  - {}", s)).size(12.0).color(Color32::from_rgb(80, 150, 220)));
                    }
                }
            });

            // Publish
            section_frame().show(&mut cols[1], |ui| {
                ui.label(RichText::new(t.publish).size(14.0).strong().color(Color32::from_rgb(200, 210, 225)));
                ui.add_space(6.0);
                input_field(ui, t.topic, &mut self.pub_topic);
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t.qos).size(13.0).color(Color32::from_rgb(180, 185, 195)));
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_salt("qos").selected_text(format!("{}", self.pub_qos)).width(40.0).show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.pub_qos, 0, "0");
                        ui.selectable_value(&mut self.pub_qos, 1, "1");
                        ui.selectable_value(&mut self.pub_qos, 2, "2");
                    });
                });
                ui.add_space(3.0);
                input_field(ui, t.msg, &mut self.pub_msg);
                ui.add_space(6.0);
                if ui.small_button(RichText::new(t.pub_btn).size(13.0)).clicked() && !self.pub_topic.is_empty() {
                    let qos = match self.pub_qos {
                        1 => QoS::AtLeastOnce, 2 => QoS::ExactlyOnce, _ => QoS::AtMostOnce,
                    };
                    self.manager.publish(&self.pub_topic, qos, &self.pub_msg);
                }
            });
        });
    }

    fn ui_log(&mut self, ui: &mut egui::Ui, t: &Texts) {
        egui::Frame::NONE
            .fill(Color32::from_rgb(20, 22, 26))
            .stroke(Stroke::new(1.0, Color32::from_rgb(48, 52, 60)))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin { left: 10, right: 10, top: 8, bottom: 8 })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t.log).size(14.0).strong().color(Color32::from_rgb(200, 210, 225)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(t.clear).clicked() { self.manager.clear_logs(); }
                    });
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical().max_height(190.0).stick_to_bottom(true).show(ui, |ui| {
                    for msg in self.manager.logs() {
                        match msg {
                            MqttLog::Info(s) => { ui.label(RichText::new(format!("[{}] {}", t.info, s)).size(12.0).color(Color32::from_rgb(215, 190, 85))); }
                            MqttLog::Recv { topic, payload } => { ui.label(RichText::new(format!("[{}] [{}] {}", t.recv, topic, payload)).size(12.0).color(Color32::from_rgb(80, 195, 125))); }
                            MqttLog::Send { topic, payload } => { ui.label(RichText::new(format!("[{}] [{}] {}", t.send, topic, payload)).size(12.0).color(Color32::from_rgb(80, 155, 220))); }
                            MqttLog::Error(s) => { ui.label(RichText::new(format!("[{}] {}", t.err, s)).size(12.0).color(Color32::from_rgb(220, 85, 85))); }
                        }
                    }
                });
            });
    }

    fn ui_statusbar(&mut self, ui: &mut egui::Ui, t: &Texts) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            let (clr, txt) = if self.manager.connected() {
                (Color32::from_rgb(65, 190, 105), t.connected)
            } else {
                (Color32::from_rgb(110, 115, 128), t.disconnected)
            };
            let pos = ui.cursor().min + egui::vec2(5.0, 7.0);
            ui.painter().circle_filled(pos, 4.0, clr);
            ui.add_space(14.0);
            ui.label(RichText::new(txt).size(12.0).color(Color32::from_rgb(155, 160, 175)));
        });
    }

    fn save_config(&self) {
        let cfg = MqttConfig {
            host: self.host.clone(), port: self.port.clone(), path: self.path.clone(),
            username: self.username.clone(), password: self.password.clone(),
            transport: if self.transport == TransportType::Ws { "ws".into() } else { "tcp".into() },
        };
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            let path = config_file_path();
            if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
            let _ = std::fs::write(&path, &json);
            if let Some(p) = rfd::FileDialog::new().add_filter("JSON", &["json"]).set_file_name("mqtt_config.json").save_file() {
                let _ = std::fs::write(p, &json);
            }
        }
    }

    fn load_config(&mut self) {
        if let Some(p) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
            if let Ok(data) = std::fs::read_to_string(p) {
                if let Ok(c) = serde_json::from_str::<MqttConfig>(&data) {
                    self.host = c.host; self.port = c.port; self.path = c.path;
                    self.username = c.username; self.password = c.password;
                    self.transport = if c.transport == "ws" { TransportType::Ws } else { TransportType::Tcp };
                }
            }
        }
    }
}

fn config_file_path() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap_or_default();
    p.pop(); p.push("mqtt_config.json"); p
}
