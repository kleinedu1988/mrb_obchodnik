mod db_sync;

use std::thread;
use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    
    // --- 1. INICIALIZACE PŘI STARTU ---
    let (msg, status, sync_time) = db_sync::get_current_status();
    ui.set_stav_text(msg.into());
    ui.set_db_status_code(status);
    ui.set_posledni_sync(sync_time.into());

    let ui_handle = ui.as_weak();

    // --- 2. LOGIKA TLAČÍTKA (AKTUALIZACE) ---
    ui.on_vybrat_soubor_databaze(move || {
        let ui = ui_handle.unwrap();
        
        // Spustíme dialog pro výběr souboru (v hlavním vlákně, rfd to tak vyžaduje)
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel", &["xlsx", "xlsm"])
            .set_title("Vyberte soubor: Obchodní partneři.xlsx")
            .pick_file() 
        {
            // Příprava pro vlákno
            let thread_ui_handle = ui.as_weak();
            let path_clone = path.clone();

            // Aktivujeme Progress Bar v UI před startem vlákna
            ui.set_show_progress(true);
            ui.set_progress_value(0.0);
            ui.set_progress_label("Inicializace importu...".into());

            // --- VÝKONNÉ VLÁKNO ---
thread::spawn(move || {
    // Definujeme callback pro aktualizaci progressu
    let progress_callback = {
        let ui_inner = thread_ui_handle.clone();
        move |val: f32, label: &str| {
            let label_string = label.to_string();
            // KLÍČOVÁ OPRAVA: Vytvoříme kopii pro invoke_from_event_loop
            let ui_for_loop = ui_inner.clone(); 
            
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_for_loop.upgrade() {
                    ui.set_progress_value(val);
                    ui.set_progress_label(label_string.into());
                }
            });
        }
    };

    // Nyní už progress_callback implementuje FnMut a kód půjde zkompilovat
    let vysledek = db_sync::handle_database_update_with_progress(path_clone, progress_callback);

                // Po skončení vlákna aktualizujeme finální stav v UI
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = thread_ui_handle.upgrade() {
                        ui.set_show_progress(false); // Schováme progress bar
                        
                        if let Some((msg, status)) = vysledek {
                            ui.set_stav_text(msg.into());
                            ui.set_db_status_code(status);
                            
                            // Načteme nový čas synchronizace
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