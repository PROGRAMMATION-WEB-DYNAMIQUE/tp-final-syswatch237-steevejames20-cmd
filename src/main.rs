use std::{
    fmt,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::Local;
use sysinfo::System;

#[derive(Debug, Clone)]
struct CpuInfo {
    usage: f32,
}

#[derive(Debug, Clone)]
struct MemInfo {
    used_mb: u64,
    total_mb: u64,
}

#[derive(Debug, Clone)]
struct ProcessInfo {
    name: String,
    cpu: f32,
}

#[derive(Debug, Clone)]
struct SystemSnapshot {
    cpu: CpuInfo,
    mem: MemInfo,
    processes: Vec<ProcessInfo>,
}

impl fmt::Display for SystemSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f,"CPU: {:.2}%", self.cpu.usage)?;
        writeln!(f,"RAM: {} MB / {} MB", self.mem.used_mb, self.mem.total_mb)?;
        writeln!(f,"Top processes:")?;
        for p in &self.processes {
            writeln!(f,"- {} ({:.2}%)", p.name, p.cpu)?;
        }
        Ok(())
    }
}

fn collect_snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut procs: Vec<ProcessInfo> = sys.processes()
        .values()
        .map(|p| ProcessInfo {
            // Correction Windows/sysinfo 0.30
            name: p.name().to_string(),

            cpu: p.cpu_usage(),
        })
        .collect();

    // Trier les processus par charge CPU
    procs.sort_by(|a,b| b.cpu.partial_cmp(&a.cpu).unwrap());
    procs.truncate(5);

    SystemSnapshot{
        // Correction API sysinfo 0.30
        cpu: CpuInfo {
            usage: sys.global_cpu_info().cpu_usage()
        },

        mem: MemInfo{
            used_mb: sys.used_memory()/1024,
            total_mb: sys.total_memory()/1024,
        },

        processes: procs,
    }
}

fn bar(v:f32)->String{
    let filled=(v/10.0).round() as usize;
    format!("[{}{}] {:.1}%",
        "█".repeat(filled.min(10)),
        "-".repeat(10-filled.min(10)),
        v)
}

fn format_response(s:&SystemSnapshot, cmd:&str)->String{
    match cmd.trim() {
        "cpu" => format!("CPU {}\n", bar(s.cpu.usage)),
        "mem" => format!("RAM {} / {} MB\n", s.mem.used_mb, s.mem.total_mb),
        "ps" => {
            let mut out=String::from("Top processes:\n");
            for p in &s.processes {
                out.push_str(&format!("{} {:.2}%\n",p.name,p.cpu));
            }
            out
        }
        "all" => format!("{}\n", s),
        "help" => "Commands: cpu mem ps all help quit\n".into(),
        "quit" => "Bye.\n".into(),
        _ => "Unknown command. type help\n".into()
    }
}

fn log_event(msg:&str){
    let mut f=OpenOptions::new().create(true).append(true).open("syswatch.log").unwrap();
    writeln!(f,"[{}] {}",Local::now().format("%Y-%m-%d %H:%M:%S"),msg).ok();
}

fn handle_client(mut stream:TcpStream, shared:Arc<Mutex<SystemSnapshot>>){
    let peer=stream.peer_addr().ok();
    log_event(&format!("Client connected {:?}",peer));
    stream.write_all(b"Welcome to SysWatch. type help\n").ok();

    let clone=stream.try_clone().unwrap();
    let mut reader=BufReader::new(clone);

    loop{
        stream.write_all(b"> ").ok();
        let mut line=String::new();
        if reader.read_line(&mut line).is_err(){break;}
        if line.trim().is_empty(){continue;}

        log_event(&format!("Command {}", line.trim()));
        let snap=shared.lock().unwrap().clone();
        let resp=format_response(&snap,&line);
        stream.write_all(resp.as_bytes()).ok();

        if line.trim()=="quit" { break; }
    }
}

fn main() {
    let shared=Arc::new(Mutex::new(collect_snapshot()));

    let updater=shared.clone();
    thread::spawn(move ||{
        loop{
            {
                let mut data=updater.lock().unwrap();
                *data=collect_snapshot();
            }
            thread::sleep(Duration::from_secs(5));
        }
    });

    let listener=TcpListener::bind("127.0.0.1:7878").expect("bind failed");
    println!("SysWatch running on port 7878");
    println!("Connect with: telnet 127.0.0.1 7878");

    for stream in listener.incoming(){
        if let Ok(stream)=stream{
            let data=shared.clone();
            thread::spawn(move || handle_client(stream,data));
        }
    }
}
