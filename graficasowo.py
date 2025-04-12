import pandas as pd #Para el manejo y análisis de datos 
import matplotlib.pyplot as plt #Crea los grafos
import json 
from datetime import datetime #Fechas y horas
from pathlib import Path #Rutas
import tkinter as tk #Interfáz
from tkinter import ttk
# Leer archivo JSONL
data = []
with open("datosuwu.jsonl", "r", encoding="utf-8") as f:
    for line in f:
        data.append(json.loads(line))
# Convertir a DataFrame
df = pd.DataFrame(data)
df["timestamp"] = pd.to_datetime(df["timestamp"], format="%Y-%m-%d %H:%M:%S")
df = df.sort_values("timestamp")
# Los colorsitos chulos uvu
rosa_pastel = '#FFB6C1'
rosa_medio = '#FF69B4'
rosa_fuerte = '#FF1493'
rosa_oscuro = '#C71585'
rosa_violeta = '#DB7093'
# Gráfica del CPU
plt.figure(figsize=(10, 5))
plt.fill_between(df["timestamp"], df["cpu_total_usage"], color=rosa_pastel, alpha=0.6)
plt.plot(df["timestamp"], df["cpu_total_usage"], color=rosa_oscuro, linewidth=2)
plt.title("USO DEL CPU %", fontweight='bold', color=rosa_oscuro)
plt.xlabel("TIEMPO")
plt.ylabel("CPU %")
plt.xticks(rotation=45)
plt.grid(True, linestyle='--', alpha=0.4, color=rosa_violeta)
plt.tight_layout()
plt.show()
# Gráfica de la Memoria
plt.figure(figsize=(10, 5))
bar_width = 0.02
plt.bar(df["timestamp"], df["used_memory_mb"], width=bar_width, color=rosa_medio, label="Usada")
plt.bar(df["timestamp"], df["total_memory_mb"] - df["used_memory_mb"],
        bottom=df["used_memory_mb"], width=bar_width, color=rosa_pastel, label="Libre")
plt.title("Uso de Memoria (MB)", fontweight='bold', color=rosa_oscuro)
plt.xlabel("Tiempo")
plt.ylabel("Memoria (MB)")
plt.xticks(rotation=45)
plt.legend()
plt.grid(True, axis='y', linestyle='--', alpha=0.4, color=rosa_violeta)
plt.tight_layout()
plt.show()
# Gráfica de Red
plt.figure(figsize=(12, 6))
plt.plot(df["timestamp"], df["total_received_mb"], color=rosa_fuerte, linewidth=2.5, label="Red recibida (MB)")
plt.plot(df["timestamp"], df["total_transmitted_mb"], color=rosa_violeta, linewidth=2.5, linestyle='-.', label="Red enviada (MB)")
plt.fill_between(df["timestamp"], df["total_received_mb"], alpha=0.1, color=rosa_fuerte)
plt.fill_between(df["timestamp"], df["total_transmitted_mb"], alpha=0.1, color=rosa_violeta)
plt.ylabel("Red (MB)")
plt.xlabel("Tiempo")
plt.title("Uso de Red", fontweight='bold', color=rosa_oscuro)
plt.grid(True, linestyle=':', alpha=0.3, color=rosa_medio)
plt.legend()
plt.tight_layout()
plt.show()
# Gráfica del Disco
plt.figure(figsize=(12, 6))
bar_width = 0.01
plt.bar(df["timestamp"], df["disk_reads_mb"], width=bar_width, color=rosa_medio, label="Lecturas (MB)")
plt.bar(df["timestamp"], df["disk_writes_mb"], width=bar_width, color=rosa_violeta, label="Escrituras (MB)")
plt.ylabel("Disco (MB)")
plt.xlabel("Tiempo")
plt.title("Actividad de Disco", fontweight='bold', color=rosa_oscuro)
plt.grid(True, axis='y', linestyle='--', alpha=0.4, color=rosa_violeta)
plt.legend()
plt.tight_layout()
plt.show()
# Interfaz principal c:
bg_color = "#ffe4f0"
header_color = "#ffb6c1"
row_color = "#fff0f5"
text_color = "#d63384"
root = tk.Tk()
root.title("Top 5 procesos por uso de CPU")
root.geometry("700x600")
root.configure(bg=bg_color)
main_frame = tk.Frame(root, bg=bg_color)
main_frame.pack(fill=tk.BOTH, expand=True)
canvas = tk.Canvas(main_frame, bg=bg_color, highlightthickness=0)
scrollbar = tk.Scrollbar(main_frame, orient=tk.VERTICAL, command=canvas.yview)
scrollable_frame = tk.Frame(canvas, bg=bg_color)
scrollable_frame.bind("<Configure>", lambda e: canvas.configure(scrollregion=canvas.bbox("all")))
canvas.create_window((0, 0), window=scrollable_frame, anchor="nw")
canvas.configure(yscrollcommand=scrollbar.set)
canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
style = ttk.Style()
style.theme_use("clam")
style.configure("Treeview", background=row_color, fieldbackground=row_color, foreground=text_color, rowheight=25, font=("Arial", 10))
style.configure("Treeview.Heading", background=header_color, foreground="black", font=("Arial", 10, "bold"))
for _, row in df.iterrows():
    timestamp_str = row["timestamp"].strftime("%Y-%m-%d %H:%M:%S")
    procesos = row["top_cpu_processes"]
    label = tk.Label(scrollable_frame, text=timestamp_str, font=("Arial", 12, "bold"), bg=bg_color, fg=text_color)
    label.pack(anchor="w", pady=(10, 0))
    tree = ttk.Treeview(scrollable_frame, columns=("Proceso"), show="headings", height=5)
    tree.heading("Proceso", text="Proceso")
    tree.column("Proceso", anchor="w", width=600)
    for proc in procesos:
        tree.insert("", "end", values=(proc,))
    tree.pack(anchor="w", pady=(0, 20))
def abrir_ventana_analisis():
    ventana_analisis = tk.Toplevel(root)
    ventana_analisis.title("Análisis del Sistema")
    ventana_analisis.geometry("750x500")
    ventana_analisis.configure(bg=bg_color)
    texto = tk.Text(ventana_analisis, wrap="word", bg=row_color, fg=text_color, font=("Arial", 11))
    texto.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
    texto.insert(tk.END, " AN\u00c1LISIS DE USO DEL SISTEMA\n\n")
    cpu_avg = df["cpu_total_usage"].mean()
    cpu_max = df["cpu_total_usage"].max()
    mem_avg = df["used_memory_mb"].mean()
    net_max = df["total_received_mb"].max() + df["total_transmitted_mb"].max()
    texto.insert(tk.END, f" Uso promedio de CPU: {cpu_avg:.2f}%\n")
    texto.insert(tk.END, f" Pico m\u00e1ximo de CPU: {cpu_max:.2f}%\n")
    texto.insert(tk.END, f" Memoria promedio usada: {mem_avg:.2f} MB\n")
    texto.insert(tk.END, f" Mayor tr\u00e1fico de red registrado: {net_max:.2f} MB\n\n")
    corr_cpu = df.corr(numeric_only=True)["cpu_total_usage"].sort_values(ascending=False)
    traducciones = {
        "used_memory_mb": "Memoria usada (MB)",
        "disk_reads_mb": "Lecturas de disco (MB)",
        "disk_writes_mb": "Escrituras de disco (MB)",
        "total_received_mb": "Red recibida (MB)",
        "total_transmitted_mb": "Red transmitida (MB)",
        "used_swap_mb": "Swap usada (MB)",
        "total_swap_mb": "Swap total (MB)",
        "free_memory_mb": "Memoria libre (MB)",
        "cpu_frequency_mhz": "Frecuencia CPU (MHz)",
        "total_memory_mb": "Memoria total (MB)"
    }
    texto.insert(tk.END, " Correlaciones con CPU:\n")
    for metric, val in corr_cpu.items():
        if metric != "cpu_total_usage":
            nombre = traducciones.get(metric, metric) 
            texto.insert(tk.END, f" -> {nombre}: {val:.2f}\n")
    texto.insert(tk.END, "\n")
    df["chrome_activo"] = df["top_cpu_processes"].apply(lambda procs: any("chrome" in proc.lower() for proc in procs))
    if df["chrome_activo"].sum() > 0:
        chrome_cpu = df[df["chrome_activo"]]["cpu_total_usage"].mean()
        texto.insert(tk.END, f" Cuando Chrome est\u00e1 activo, el CPU promedia: {chrome_cpu:.2f}%\n\n")
    try:
        from sklearn.linear_model import LinearRegression
        df["timestamp_num"] = df["timestamp"].astype("int64") // 10**9
        X = df[["timestamp_num"]]
        y = df["used_swap_mb"]
        model = LinearRegression().fit(X, y)
        swap_total = df["total_swap_mb"].max()
        time_to_full = (swap_total - model.intercept_) / model.coef_[0]
        estimated_time = pd.to_datetime(time_to_full, unit='s')
        texto.insert(tk.END, f" Si el uso contin\u00faa igual, el swap se llenar\u00e1 el {estimated_time.strftime('%Y-%m-%d %H:%M:%S')}.\n\n")
    except Exception as e:
        texto.insert(tk.END, f" No se pudo calcular proyecci\u00f3n de swap: {str(e)}\n\n")
    dias_traducidos = {
        "Monday": "Lunes",
        "Tuesday": "Martes",
        "Wednesday": "Miércoles",
        "Thursday": "Jueves",
        "Friday": "Viernes",
        "Saturday": "Sábados",
        "Sunday": "Domingos"
    }
    df["dia_semana"] = df["timestamp"].dt.day_name()
    df["dia_semana"] = df["dia_semana"].map(dias_traducidos)
    df["hora"] = df["timestamp"].dt.hour
    uso_red_por_hora_dia = df.groupby(["dia_semana", "hora"])[["total_received_mb", "total_transmitted_mb"]].mean()
    uso_red_total = uso_red_por_hora_dia.sum(axis=1)
    peak_uso = uso_red_total.idxmax()
    texto.insert(tk.END, f" El mayor uso promedio de red ocurre los *{peak_uso[0]} a las {peak_uso[1]}:00 hrs*\n")
    texto.configure(state="disabled")
boton_analisis = tk.Button(root, text=" Ver an\u00e1lisis del sistema", command=abrir_ventana_analisis,
                           font=("Arial", 11, "bold"), bg=header_color, fg="black", relief="raised")
boton_analisis.pack(pady=10)
root.mainloop()
