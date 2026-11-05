use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tuhc_lib::config::{AppConfig, Tab};

#[test]
fn test_concurrent_config_save_and_load() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    // Ensure initial config is saved
    let initial = AppConfig::default();
    initial.save().expect("Initial save must succeed");

    let stop = Arc::new(AtomicBool::new(false));
    let save_success = Arc::new(AtomicUsize::new(0));
    let load_success = Arc::new(AtomicUsize::new(0));
    let load_errors = Arc::new(AtomicUsize::new(0));
    let disk_corruptions = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    // 6 writer threads rapidly modifying and saving config
    for thread_id in 0..6 {
        let stop_clone = Arc::clone(&stop);
        let save_cnt = Arc::clone(&save_success);
        let handle = std::thread::spawn(move || {
            let mut iter = 0;
            while !stop_clone.load(Ordering::Relaxed) && iter < 100 {
                let mut cfg = AppConfig::default();
                cfg.tab_data.active_tab_key = format!("tab_{}_{}", thread_id, iter);
                cfg.settings
                    .mod_list_enabled
                    .push(format!("mod_{}", thread_id));

                let tab = Tab {
                    key: format!("tab_{}_{}", thread_id, iter),
                    url: format!("/page/{}", iter),
                    title: format!("Title {} {}", thread_id, iter),
                    has_embed: false,
                    history: vec!["001901".to_string(), "001902".to_string()],
                    future: vec![],
                };
                cfg.tab_data
                    .tabs
                    .insert(format!("tab_{}_{}", thread_id, iter), tab);

                match cfg.save() {
                    Ok(_) => {
                        save_cnt.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        eprintln!("[Thread {}] save error: {}", thread_id, e);
                    }
                }
                iter += 1;
                std::thread::yield_now();
            }
        });
        handles.push(handle);
    }

    // 4 reader threads continuously reading config and validating JSON integrity
    let config_file_path = AppConfig::config_path();
    for thread_id in 0..4 {
        let stop_clone = Arc::clone(&stop);
        let load_cnt = Arc::clone(&load_success);
        let err_cnt = Arc::clone(&load_errors);
        let corrupt_cnt = Arc::clone(&disk_corruptions);
        let cfg_path = config_file_path.clone();

        let handle = std::thread::spawn(move || {
            while !stop_clone.load(Ordering::Relaxed) {
                // 1. Direct AppConfig::load() call
                let cfg = AppConfig::load();
                if cfg.app_version != "2.8.1" {
                    err_cnt.fetch_add(1, Ordering::Relaxed);
                } else {
                    load_cnt.fetch_add(1, Ordering::Relaxed);
                }

                // 2. Direct raw disk read to detect partial or corrupted file writes
                if let Ok(raw) = std::fs::read_to_string(&cfg_path) {
                    if !raw.trim().is_empty() {
                        if let Err(e) = serde_json::from_str::<serde_json::Value>(&raw) {
                            eprintln!(
                                "[Reader {}] Raw disk corruption detected in config.json: {}",
                                thread_id, e
                            );
                            corrupt_cnt.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }

                std::thread::yield_now();
            }
        });
        handles.push(handle);
    }

    // Wait for all writer threads (first 6 handles)
    for handle in handles.drain(0..6) {
        handle.join().unwrap();
    }

    // Stop reader threads
    stop.store(true, Ordering::Relaxed);
    for handle in handles {
        handle.join().unwrap();
    }

    let saves = save_success.load(Ordering::Relaxed);
    let loads = load_success.load(Ordering::Relaxed);
    let errs = load_errors.load(Ordering::Relaxed);
    let corruptions = disk_corruptions.load(Ordering::Relaxed);

    println!(
        "Config concurrency results: saves={}, loads={}, load_errors={}, raw_corruptions={}",
        saves, loads, errs, corruptions
    );

    // Verify final state of config file
    let final_raw = std::fs::read_to_string(&config_file_path).expect("Final config must exist");
    let final_parsed: serde_json::Value =
        serde_json::from_str(&final_raw).expect("Final config must be valid JSON");
    assert_eq!(
        final_parsed.get("appVersion").and_then(|v| v.as_str()),
        Some("2.8.1")
    );

    assert_eq!(errs, 0, "No load errors should occur");
    assert_eq!(corruptions, 0, "No raw disk corruptions should occur");
}
