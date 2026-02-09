mod db_sync;
mod config;

use std::thread;
use slint::ComponentHandle;
use config::AppConfig;
use std::sync::{Arc, Mutex};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    
    // Načtení konfigurace (cest) z config.json
    let config = Arc::new(Mutex::new(AppConfig::load()));

    // Inicializace cest v UI při startu aplikace
    {
        let cfg = config.lock().unwrap();
        ui.set_path_tech_docs(cfg.path_tech_docs.clone().into());
        ui.set_path_production(cfg.path_production.clone().into());
        ui.set_path_offers(cfg.path_offers.clone().into());
    }

    // Inicializace stavu databáze (původní kód)
    let (msg, status, sync_time) = db_sync::get_current_status();
    ui.set_stav_text(msg.into());
    ui.set_db_status_code(status);
    ui.set_posledni_sync(sync_time.into());

    let ui_handle = ui.as_weak();

    // --- LOGIKA PRO ZMĚNU CEST (Záložka Systém) ---
    
    // 1. Cesta k technické dokumentaci
    let cfg_c = config.clone();
    let h1 = ui_handle.clone();
    ui.on_change_tech_docs(move || {
        if let Some(path) = rfd::FileDialog::new().set_title("Vyberte složku pro dokumentaci").pick_folder() {
            let p = path.display().to_string();
            let ui = h1.unwrap();
            ui.set_path_tech_docs(p.clone().into());
            
            let mut cfg = cfg_c.lock().unwrap();
            cfg.path_tech_docs = p;
            cfg.save();
        }
    });

    // 2. Cesta do výroby
    let cfg_c = config.clone();
    let h2 = ui_handle.clone();
    ui.on_change_production(move || {
        if let Some(path) = rfd::FileDialog::new().set_title("Vyberte složku pro výrobu").pick_folder() {
            let p = path.display().to_string();
            let ui = h2.unwrap();
            ui.set_path_production(p.clone().into());
            
            let mut cfg = cfg_c.lock().unwrap();
            cfg.path_production = p;
            cfg.save();
        }
    });

    // 3. Cesta k nabídkám
    let cfg_c = config.clone();
    let h3 = ui_handle.clone();
    ui.on_change_offers(move || {
        if let Some(path) = rfd::FileDialog::new().set_title("Vyberte složku pro nabídky").pick_folder() {
            let p = path.display().to_string();
            let ui = h3.unwrap();
            ui.set_path_offers(p.clone().into());
            
            let mut cfg = cfg_c.lock().unwrap();
            cfg.path_offers = p;
            cfg.save();
        }
    });

    // --- LOGIKA IMPORTU DATABÁZE (Progress Bar) ---
    let h_import = ui_handle.clone();
    ui.on_vybrat_soubor_databaze(move || {
        let ui = h_import.unwrap();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel", &["xlsx", "xlsm"])
            .pick_file() 
        {
            let thread_ui_handle = ui.as_weak();
            let path_clone = path.clone();

            ui.set_show_progress(true);
            ui.set_progress_value(0.0);
            ui.set_progress_label("Inicializace importu...".into());

            thread::spawn(move || {
                let progress_callback = {
                    let ui_inner = thread_ui_handle.clone();
                    move |val: f32, label: &str| {
                        let label_string = label.to_string();
                        let ui_for_loop = ui_inner.clone(); 
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_for_loop.upgrade() {
                                ui.set_progress_value(val);
                                ui.set_progress_label(label_string.into());
                            }
                        });
                    }
                };

                let vysledek = db_sync::handle_database_update_with_progress(path_clone, progress_callback);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = thread_ui_handle.upgrade() {
                        ui.set_show_progress(false);
                        if let Some((msg, status)) = vysledek {
                            ui.set_stav_text(msg.into());
                            ui.set_db_status_code(status);
                            let (_, _, new_sync) = db_sync::get_current_status();
                            ui.set_posledni_sync(new_sync.into());
                        }
                    }
                });
            });
        }
    });

    ui.run()
}