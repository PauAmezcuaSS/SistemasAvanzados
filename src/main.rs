//PAULINA AMEZCUA GARCÍA 09/04/24 Monitor de sistema personalizado
use sysinfo::{System, Networks, Components}; 
//System permite acceder a información general del sistema 
//Networks permite acceder a la información de las interfaces de red del sistema 
//Components permite acceder a información de los componentes físicos
use std::fs::File;
use std::io::Write; //Para importar el trait Write del módulo std::io (entrada/salida estándar)
use std::fs::OpenOptions; //Importa la estructura OpenOptions del módulo std::fs (sistema de archivos) 
use tokio; //Es una runtime (entorno de ejecución) 
use serde::Serialize; //Serialización y deserialización de datos
use chrono::Local; //Para manejar fechas y horas.
use heim::disk; //Para obtener información sobre el hardware y el sistema 
use futures::stream::StreamExt; //Para trabajar con futuros y flujos 
use heim::units::information::byte; //Para obtener información sobre el sistema y hardware 
#[tokio::main] //Se ejecutará dentro del entorno de ejecución de Tokio, que es un runtime 
async fn main() {
    let mut system = System::new_all();
    system.refresh_all();
    // Para obtener la información de CPU
    let cpu_total_usage = system.global_cpu_info().cpu_usage();
    let cpu_frequency_mhz = system.global_cpu_info().frequency();
    // Para obtener el uso por núcleo
    let cpu_cores_usage: Vec<String> = system
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| format!("Core {}: {:.2}%", i, cpu.cpu_usage()))
        .collect();
    // Para obtener la información de memoria
    let used_memory_mb = system.used_memory() / 1024;
    let total_memory_mb = system.total_memory() / 1024;
    let used_swap_mb = system.used_swap() / 1024;
    let total_swap_mb = system.total_swap() / 1024;
    let free_memory_mb = system.free_memory() / 1024;
    // Para obtener la información de red
    let networks = Networks::new_with_refreshed_list();
    let mut total_received_mb = 0.0;
    let mut total_transmitted_mb = 0.0;
    for (_, data) in &networks {
        total_received_mb += data.total_received() as f64 / (1024.0 * 1024.0);
        total_transmitted_mb += data.total_transmitted() as f64 / (1024.0 * 1024.0);
    }
    // Para obtener la temperatura de componentes
    let components = Components::new_with_refreshed_list();
    let component_temperatures: Vec<String> = components
        .iter()
        .map(|c| format!("{}: {:.2}°C", c.label(), c.temperature()))
        .collect();
    // Para obtener la información de disco de forma asíncrona
    let mut disk_reads_mb = 0.0;
    let mut disk_writes_mb = 0.0;
    let mut disk_stream = disk::io_counters().await.unwrap();
    while let Some(Ok(disk)) = disk_stream.next().await {
        disk_reads_mb += disk.read_bytes().get::<byte>() as f64 / (1024.0 * 1024.0);
        disk_writes_mb += disk.write_bytes().get::<byte>() as f64 / (1024.0 * 1024.0);
    }
    // Para obtener los top 5 procesos que más consumen
    let mut processes: Vec<_> = system.processes().values().collect();
    processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
    let top_cpu_processes: Vec<String> = processes.iter().take(5).map(|procesouvu| {
        format!(
            "{}: {:.2}% CPU, {} KB memoria",
            procesouvu.name(),
            procesouvu.cpu_usage(),
            procesouvu.memory()
        )
    }).collect();
    #[derive(Serialize)]
    //Aquí se define la estructura datosuwu
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
        total_received_mb: f64,
        total_transmitted_mb: f64,
        disk_reads_mb: f64,
        disk_writes_mb: f64,
        component_temperatures: Vec<String>,
        top_cpu_processes: Vec<String>,
    }
    //Aquí se define la instancia Datosuwu
    let datosuwu = Datosuwu {
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
        component_temperatures,
        top_cpu_processes,
    };
    //Se convierte de datosuwu a Json 
    let json_line = serde_json::to_string(&datosuwu).unwrap();
    //Se abre el archivo y se escribe el Json
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("datosuwu.jsonl")
        .unwrap();
    writeln!(file, "{}", json_line).unwrap();
}