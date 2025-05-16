use eframe::{egui, App, Frame}; 
use sysinfo::{System, Networks};
use serde::Serialize;
use chrono::Local;
use futures::stream::StreamExt;
use heim::disk;
use heim::units::information::byte;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

#[derive(Default)]
struct MetricsApp {
    datos: Arc<Mutex<Option<Datosuwu>>>,
    last_update: Option<Instant>,
}

#[derive(Serialize, Clone)]
struct Datosuwu {
    timestamp: String,
    cpu_total_usage: f32,
    cpu_frequency_mhz: u64,
    cpu_cores_usage: Vec<String>,
    used_memory_mb: u64,
    total_memory_mb: u64,
    used_swap_mb: u64,
    total_swap_mb: u64,
    free_memory_mb: u64,
    total_received_mb: u64,
    total_transmitted_mb: u64,
    disk_reads_mb: f64,
    disk_writes_mb: f64,
    top_cpu_processes: Vec<String>,
}

impl MetricsApp {
    async fn obtener_datos() -> Datosuwu {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu_total_usage = system.global_cpu_info().cpu_usage();
        let cpu_frequency_mhz = system.global_cpu_info().frequency();
        let cpu_cores_usage: Vec<String> = system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| format!("Core {}: {:.2}%", i, cpu.cpu_usage()))
            .collect();

        let used_memory_mb = system.used_memory() / 1024;
        let total_memory_mb = system.total_memory() / 1024;
        let used_swap_mb = system.used_swap() / 1024;
        let total_swap_mb = system.total_swap() / 1024;
        let free_memory_mb = system.free_memory() / 1024;
        
        let networks = Networks::new_with_refreshed_list();
        let mut total_received_mb = 0;
        let mut total_transmitted_mb = 0;

        for (_, data) in &networks {
            total_received_mb += data.total_received() / (1024 * 1024);
            total_transmitted_mb += data.total_transmitted() / (1024 * 1024);
        }

        let mut disk_reads_mb = 0.0;
        let mut disk_writes_mb = 0.0;
        let mut disk_stream = disk::io_counters().await.unwrap();
        while let Some(Ok(disk)) = disk_stream.next().await {
            disk_reads_mb += disk.read_bytes().get::<byte>() as f64 / (1024.0 * 1024.0);
            disk_writes_mb += disk.write_bytes().get::<byte>() as f64 / (1024.0 * 1024.0);
        }

        let mut processes: Vec<_> = system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
        let top_cpu_processes: Vec<String> = processes.iter().take(5).map(|p| {
            format!("{}: {:.2}% CPU, {} KB memoria", p.name(), p.cpu_usage(), p.memory())
        }).collect();

        Datosuwu {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            cpu_total_usage,
            cpu_frequency_mhz,
            cpu_cores_usage,
            used_memory_mb,
            total_memory_mb,
            used_swap_mb,
            total_swap_mb,
            free_memory_mb,
            total_received_mb,
            total_transmitted_mb,
            disk_reads_mb,
            disk_writes_mb,
            top_cpu_processes,
        }
    }
}

impl App for MetricsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Verificar si es hora de actualizar
        let should_update = match self.last_update {
            Some(last) => last.elapsed() >= Duration::from_secs(5),
            None => true,
        };

        if should_update {
            let ctx = ctx.clone();
            let datos_arc = Arc::clone(&self.datos);
            
            tokio::spawn(async move {
                let nuevos_datos = MetricsApp::obtener_datos().await;
                let mut datos = datos_arc.lock().await;
                *datos = Some(nuevos_datos);
                ctx.request_repaint();
            });
            
            self.last_update = Some(Instant::now());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let datos = {
                match self.datos.try_lock() {
                    Ok(guard) => guard.clone(),
                    Err(_) => {
                        ui.label("Actualizando datos...");
                        return;
                    }
                }
            };

            let datos = match datos.as_ref() {
                Some(d) => d,
                None => {
                    ui.label("Cargando datos del sistema...");
                    return;
                }
            };


            ui.label(format!("Hora: {}", datos.timestamp));
            ui.separator();

            ui.label(format!("CPU Total: {:.2}%", datos.cpu_total_usage));
            ui.label(format!("Frecuencia CPU: {} MHz", datos.cpu_frequency_mhz));
            for core in &datos.cpu_cores_usage {
                ui.label(core);
            }

            ui.separator();
            ui.label(format!(
                "Memoria usada: {} MB / {} MB",
                datos.used_memory_mb, datos.total_memory_mb
            ));
            ui.label(format!(
                "Swap usada: {} MB / {} MB",
                datos.used_swap_mb, datos.total_swap_mb
            ));
            ui.label(format!("Memoria libre: {} MB", datos.free_memory_mb));

            ui.separator();
            ui.label(format!("Red recibida: {:.2} MB", datos.total_received_mb));
            ui.label(format!("Red enviada: {:.2} MB", datos.total_transmitted_mb));

            ui.separator();
            ui.label(format!("Lecturas de disco: {:.2} MB", datos.disk_reads_mb));
            ui.label(format!("Escrituras de disco: {:.2} MB", datos.disk_writes_mb));

            ui.separator();
            ui.label("Top procesos por CPU:");
            for proc in &datos.top_cpu_processes {
                ui.label(proc);
            }

            if ui.button("Actualizar métricas ahora").clicked() {
                let ctx = ctx.clone();
                let datos_arc = Arc::clone(&self.datos);
                
                tokio::spawn(async move {
                    let nuevos_datos = MetricsApp::obtener_datos().await;
                    let mut datos = datos_arc.lock().await;
                    *datos = Some(nuevos_datos);
                    ctx.request_repaint();
                });
                
                self.last_update = Some(Instant::now());
            }
        });
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let datos_iniciales = MetricsApp::obtener_datos().await;
    let datos_shared = Arc::new(Mutex::new(Some(datos_iniciales)));
    
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "Monitor del Sistema UwU",
        native_options,
        Box::new(move |_cc| {
            Box::new(MetricsApp {
                datos: Arc::clone(&datos_shared),
                last_update: Some(Instant::now()),
            })
        }),
    )
}