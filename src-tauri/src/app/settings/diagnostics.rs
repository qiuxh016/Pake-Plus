use super::io::load_settings;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Clone, Debug, Serialize)]
pub struct Diagnostics {
    pub app_version: String,
    pub git_commit: String,
    pub build_time: String,
    pub rustc_version: String,
    pub target_triple: String,
    pub os_name: String,
    pub os_version: String,
    pub cpu_cores: usize,
    pub total_ram_mb: u64,
    pub used_ram_mb: u64,
    pub disk_total_mb: u64,
    pub disk_free_mb: u64,
    pub pid: u32,
    pub enabled_features: Vec<String>,
}

pub fn collect_diagnostics(app: &AppHandle) -> Diagnostics {
    let settings = load_settings(app);

    let mut enabled_features = Vec::new();
    if settings.adblock.enabled {
        enabled_features.push("adblock".into());
    }
    if settings.cache.enabled {
        enabled_features.push("cache".into());
    }
    if settings.clipboard.enabled {
        enabled_features.push("clipboard".into());
    }
    enabled_features.push("data-export".into());

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let os_name = sysinfo::System::name().unwrap_or_else(|| "Unknown".into());
    let os_version = sysinfo::System::os_version().unwrap_or_else(|| "Unknown".into());
    let cpu_cores = sys.cpus().len();
    let total_ram_mb = sys.total_memory() / 1024 / 1024;
    let used_ram_mb = sys.used_memory() / 1024 / 1024;
    std::mem::drop(sys);

    let disks = sysinfo::Disks::new_with_refreshed_list();
    let (disk_total_mb, disk_free_mb) = disks
        .list()
        .first()
        .map(|d| {
            (
                d.total_space() / 1024 / 1024,
                d.available_space() / 1024 / 1024,
            )
        })
        .unwrap_or((0, 0));

    mod built_info {
        include!(concat!(env!("OUT_DIR"), "/built.rs"));
    }

    Diagnostics {
        app_version: built_info::PKG_VERSION.into(),
        git_commit: built_info::GIT_COMMIT_HASH.unwrap_or("unknown").into(),
        build_time: chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        rustc_version: built_info::RUSTC_VERSION.into(),
        target_triple: built_info::TARGET.into(),
        os_name,
        os_version,
        cpu_cores,
        total_ram_mb,
        used_ram_mb,
        disk_total_mb,
        disk_free_mb,
        pid: std::process::id(),
        enabled_features,
    }
}

pub fn get_diagnostics_report(app: &AppHandle) -> String {
    let d = collect_diagnostics(app);
    format!(
        "Pake Plus v{ver} (commit: {commit})\n\
         built: {built_time}, rustc {rustc}\n\
         target: {target}\n\
         OS: {os} {os_ver}\n\
         CPU: {cpu} cores, RAM: {used_ram}/{total_ram} MB\n\
         Disk: {disk_free}/{disk_total} MB free\n\
         Enabled: {features}\n\
         PID: {pid}",
        ver = d.app_version,
        commit = d.git_commit,
        built_time = d.build_time,
        rustc = d.rustc_version,
        target = d.target_triple,
        os = d.os_name,
        os_ver = d.os_version,
        cpu = d.cpu_cores,
        used_ram = d.used_ram_mb,
        total_ram = d.total_ram_mb,
        disk_free = d.disk_free_mb,
        disk_total = d.disk_total_mb,
        features = d.enabled_features.join(", "),
        pid = d.pid,
    )
}
