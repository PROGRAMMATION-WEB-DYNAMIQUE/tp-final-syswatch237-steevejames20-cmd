# SysWatch (Rust TCP System Monitor)

Projet complet TP Rust:
- Monitoring CPU / RAM / top processes
- Serveur TCP multi-thread
- Arc<Mutex<SystemSnapshot>>
- Journalisation syswatch.log

## Lancer
```bash
cargo build
cargo run
```

Dans un autre terminal:

```bash
telnet 127.0.0.1 7878
```
Si telnet n'est pas reconnu par windows, il faut l'activer via powershell(en admin)

```bash
dism /online /Enable-Feature /FeatureName:TelnetClient
```
Puis relancer dans le second terminal vscode

```bash
telnet 127.0.0.1 7878
```

## Commandes
- cpu
- mem
- ps
- all
- help
- quit

## Concepts Rust utilisés
- Struct / Enum-ready modeling
- Traits Display
- Result-like error patterns
- Threads
- Mutex + Arc
- File logging
