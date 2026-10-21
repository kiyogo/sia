# Online Backup & Disaster Recovery

## 1. Online SQLite Backup API

Traditional file copying of a live SQLite database during active writes can cause corruption, partial page writes, or torn transactions.

The Workspace leverages SQLite's native **Online Backup API** (`rusqlite::backup::Backup` with the `backup` feature enabled in `Cargo.toml`).

### Characteristics
- **Non-Blocking:** Readers and writers continue operating without holding exclusive locks on the database file.
- **Consistent Snapshot:** Produces an exact point-in-time snapshot of the database, including all WAL pages.
- **Progress Tracking:** Pages are copied incrementally with step-wise progress monitoring.

## 2. Backup Execution

```rust
// Invoked via Rust or IPC command 'backup_database_now'
let backup_path = PathBuf::from("C:\\workspace\\backups\\backup_20261021_120000.db");
db.backup_to(&backup_path).await?;
```

Via PowerShell:
```powershell
.\scripts\backup-database.ps1
```

## 3. Disaster Recovery & Restoration

Restoring a corrupted or historical database requires:
1. Stopping the `AAIRuntime` Windows service to release active file locks.
2. Generating a safety copy of the current `data.db`.
3. Overwriting `data.db` with the validated backup file.
4. Restarting the Windows service.

Automated via PowerShell:
```powershell
.\scripts\restore-database.ps1 -BackupFile "C:\workspace\backups\backup_20261021_120000.db"
```
Verification in `test_p6_30_database_restore_verification` proves that all tables, indexes, scheduled jobs, and workflow histories remain 100% intact after restoration.
