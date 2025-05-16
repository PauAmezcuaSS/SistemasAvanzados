use eframe::egui;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::Command;
use serde::Deserialize;
use sysinfo::{System, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Deserialize, Debug, Clone)]
struct Metrics {
    cpu: u32,
    network: String,
    memory: u32,
}

enum Theme {
    Dark,
    Pink,
}

struct DefenseApp {
    received_metrics: Arc<Mutex<Vec<Metrics>>>,
    logs: Arc<Mutex<Vec<String>>>,
    last_ddos_count: Arc<Mutex<usize>>,
    current_theme: Theme,
    monitoring_active: Arc<AtomicBool>,
}

impl DefenseApp {
    fn new() -> Self {
        let metrics = Arc::new(Mutex::new(Vec::new()));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let last_ddos_count = Arc::new(Mutex::new(0));
        let monitoring_active = Arc::new(AtomicBool::new(true));

        let metrics_clone = Arc::clone(&metrics);
        let logs_clone = Arc::clone(&logs);
        let monitoring_active_clone = Arc::clone(&monitoring_active);

        let mut last_memory_values = vec![];
        let mut last_network_values: Vec<u32> = Vec::new();

        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:4000").expect("No se pudo abrir el socket UDP");
            let mut buf = [0; 1024];

            loop {
                // Verificar si el monitoreo está activo
                if !monitoring_active_clone.load(Ordering::Relaxed) {
                    thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }

                if let Ok((amt, _)) = socket.recv_from(&mut buf) {
                    if let Ok(text) = std::str::from_utf8(&buf[..amt]) {
                        if let Ok(parsed) = serde_json::from_str::<Metrics>(text) {
                            metrics_clone.lock().unwrap().push(parsed.clone());

                            // Detección de DDoS
                            if let Some(mb_str) = parsed.network.strip_suffix(" MB/s") {
                                if let Ok(mb_val) = mb_str.parse::<u32>() {
                                    last_network_values.push(mb_val);
                                    if last_network_values.len() > 5 {
                                        last_network_values.remove(0);
                                    }

                                    let ddos_spikes = last_network_values.iter().filter(|&&v| v > 70).count();

                                    if ddos_spikes >= 3 {
                                        logs_clone.lock().unwrap().push(format!(
                                            "Posible DDoS detectado: 3+ picos > 70 MB/s en ventana reciente ({:?})",
                                            last_network_values
                                        ));
                                        buscar_y_matar_proceso_sospechoso(20.0, 50);
                                    }
                                }
                            }
                            
                            // Detección de fuga de memoria
                            last_memory_values.push(parsed.memory);
                            if last_memory_values.len() > 5 {
                                last_memory_values.remove(0);
                            }

                            if last_memory_values.len() >= 4 {
                                let mut total_increase = 0;
                                let mut large_jumps = 0;

                                for win in last_memory_values.windows(2) {
                                    if win[1] > win[0] {
                                        let diff = win[1] - win[0];
                                        total_increase += diff;
                                        if diff > 10 {
                                            large_jumps += 1;
                                        }
                                    }
                                }

                                if total_increase >= 20 || large_jumps >= 2 {
                                    logs_clone.lock().unwrap().push(format!(
                                        "Posible fuga de memoria: uso de RAM subió {}% en ventana reciente {:?}",
                                        total_increase, last_memory_values
                                    ));
                                    buscar_y_matar_proceso_sospechoso(10.0, 100);
                                }
                            }

                            // Detección de pico de CPU
                            if parsed.cpu > 90 {
                                logs_clone.lock().unwrap().push(format!(
                                    "Pico de CPU detectado: uso actual del {}%",
                                    parsed.cpu
                                ));
                                kill_high_cpu_process();
                            }
                        }
                    }
                }
            }
        });

        Self {
            received_metrics: metrics,
            logs,
            last_ddos_count,
            current_theme: Theme::Pink,
            monitoring_active,
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        match self.current_theme {
            Theme::Dark => {
                style.visuals = egui::Visuals::dark();
                style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 40);
                style.visuals.panel_fill = egui::Color32::from_rgb(40, 40, 50);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(70, 70, 90);
                style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 90, 110);
                style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(110, 110, 130);
            },
            Theme::Pink => {
                style.visuals.window_fill = egui::Color32::from_rgb(255, 220, 240);
                style.visuals.panel_fill = egui::Color32::from_rgb(255, 220, 240);
                style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(255, 182, 193);
                style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(255, 160, 180);
                style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(255, 140, 170);
                style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(80, 0, 40);
                style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(100, 0, 50);
                style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(120, 0, 60);
            },
        }
        
        ctx.set_style(style);
    }
}

impl eframe::App for DefenseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);

        egui::SidePanel::left("panel_control").show(ctx, |ui| {
            ui.heading("Controles");
            
            if ui.button("Iniciar Monitoreo").clicked() {
                self.monitoring_active.store(true, Ordering::Relaxed);
                self.logs.lock().unwrap().push("Monitoreo activado.".to_string());
            }
            
            if ui.button("Desactivar Monitoreo").clicked() {
                self.monitoring_active.store(false, Ordering::Relaxed);
                self.logs.lock().unwrap().push("Monitoreo desactivado.".to_string());
            }
            
            ui.separator();
            ui.heading("Temas");
            
            if ui.button("Modo Oscuro").clicked() {
                self.current_theme = Theme::Dark;
                self.logs.lock().unwrap().push("Cambiado a modo oscuro".to_string());
            }
            
            if ui.button("Modo Claro").clicked() {
                self.current_theme = Theme::Pink;
                self.logs.lock().unwrap().push("Cambiado a modo claro".to_string());
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Métricas recibidas");

            ui.separator();
            ui.label("Log de eventos:");
            let logs = self.logs.lock().unwrap();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for log in logs.iter().rev().take(10) {
                    ui.label(log);
                }
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

fn buscar_y_matar_proceso_sospechoso(cpu_umbral: f32, memoria_umbral_mb: u64) {
    let mut sistema = System::new_all();
    sistema.refresh_all();

    for (_pid, proceso) in sistema.processes() {
        let uso_cpu = proceso.cpu_usage();
        let uso_memoria_mb = proceso.memory() / 1024;

        if uso_cpu > cpu_umbral || uso_memoria_mb > memoria_umbral_mb {
            let nombre = proceso.name();
            if nombre.contains("ataque") || nombre.contains("simulador") || nombre.contains("malicioso") {
                println!(
                    "Matando proceso sospechoso: {} (CPU: {:.1}%, RAM: {} MB)",
                    nombre, uso_cpu, uso_memoria_mb
                );
                matar_proceso_windows(nombre);
            }
        }
    }
}

fn matar_proceso_windows(nombre: &str) {
    let _ = Command::new("taskkill")
        .args(&["/IM", nombre, "/F"])
        .output();
}

fn kill_high_cpu_process() {
    let mut system = System::new_all();
    system.refresh_all();

    if let Some((pid, _)) = system.processes()
        .iter()
        .filter(|(_, p)| p.cpu_usage() > 90.0)
        .max_by_key(|(_, p)| (p.cpu_usage() * 100.0) as u32)
    {
        println!("Matando proceso con PID: {}", pid);
        let _ = system.process(*pid).map(|p| p.kill_with(Signal::Kill));
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Sistema de Defensa",
        options,
        Box::new(|_cc| Box::new(DefenseApp::new())),
    )
}