use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use rfd::FileDialog;
use calamine::{Reader, open_workbook, Xlsx, Data}; 
use serde::{Serialize, Deserialize};
use chrono::Local;

// --- DATOVÉ STRUKTURY ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Partner {
    pub id: String,
    pub nazev: String,
    pub ico: Option<String>,
    pub slozka: Option<String>,
    pub zmeneno: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Databaze {
    pub posledni_aktualizace: String,
    pub zaznamy: Vec<Partner>,
}

// --- POMOCNÉ FUNKCE ---

fn parse_ico(data: Option<&Data>) -> Option<String> {
    match data {
        Some(Data::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Some(Data::Int(i)) => Some(i.to_string()),
        Some(Data::Float(f)) => Some(f.to_string()),
        _ => None,
    }
}

pub fn get_current_status() -> (String, i32, String) {
    let json_path = Path::new("database.json");
    
    if let Ok(file) = File::open(json_path) {
        let reader = BufReader::new(file);
        if let Ok(db) = serde_json::from_reader::<_, Databaze>(reader) {
            return (
                "DATABÁZE JE NAČTENA".to_string(), 
                0, 
                db.posledni_aktualizace
            );
        }
        return ("DATABÁZE JE POŠKOZENA".to_string(), 2, "Chyba".to_string());
    }
    
    ("DATABÁZE NEEXISTUJE".to_string(), 2, "Neznámý".to_string())
}

// --- JÁDRO LOGIKY ---

fn run_import_logic<F>(excel_path: PathBuf, mut progress_cb: F) -> Result<(String, i32), String>
where F: FnMut(f32, &str) 
{
    progress_cb(0.1, "Otevírám Excel soubor...");
    let mut workbook: Xlsx<_> = open_workbook(&excel_path)
        .map_err(|e| format!("Chyba při otevírání Excelu: {}", e))?;
    
    let sheet_name = workbook.sheet_names().get(0)
        .ok_or("Excel soubor je prázdný")?.clone();
    
    let range = workbook.worksheet_range(&sheet_name)
        .map_err(|e| format!("Chyba při čtení listu: {}", e))?;

    progress_cb(0.2, "Načítám stávající JSON...");
    let json_path = Path::new("database.json");
    let mut existujici_db: HashMap<String, Partner> = HashMap::new();

    if json_path.exists() {
        if let Ok(file) = File::open(json_path) {
            let reader = BufReader::new(file);
            if let Ok(db) = serde_json::from_reader::<_, Databaze>(reader) {
                existujici_db = db.zaznamy.into_iter().map(|p| (p.id.clone(), p)).collect();
            }
        }
    }

    let mut pocet_novych = 0;
    let mut pocet_zmenenych = 0;
    let aktualni_cas = Local::now().format("%d.%m.%Y %H:%M").to_string();
    
    // Explicitně získáme řádky jako slice dat
    let rows: Vec<&[Data]> = range.rows().skip(1).collect();
    let total_rows = rows.len();

    for (idx, row) in rows.into_iter().enumerate() {
        // Získání ID (první sloupec)
        let id = match row.get(0) {
            Some(Data::String(s)) if !s.is_empty() => s.clone(),
            Some(Data::Int(i)) => i.to_string(),
            Some(Data::Float(f)) => f.to_string(),
            _ => continue, // Přeskočit řádek bez ID
        };

        // Získání Názvu (druhý sloupec)
        let nazev = match row.get(1) {
            Some(d) => d.to_string(),
            None => "Neznámý".to_string(),
        };

        let ico = parse_ico(row.get(2));

        // Merge přes Entry API
        match existujici_db.entry(id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let old_partner = entry.get_mut();
                let mut zmena = false;

                if old_partner.nazev != nazev {
                    old_partner.nazev = nazev;
                    zmena = true;
                }
                if old_partner.ico != ico {
                    old_partner.ico = ico;
                    zmena = true;
                }

                if zmena {
                    old_partner.zmeneno = aktualni_cas.clone();
                    pocet_zmenenych += 1;
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Partner {
                    id,
                    nazev,
                    ico,
                    slozka: None,
                    zmeneno: aktualni_cas.clone(),
                });
                pocet_novych += 1;
            }
        }

        // Progrese
        if idx % 50 == 0 || idx == total_rows - 1 {
            let p = 0.3 + (idx as f32 / total_rows as f32) * 0.6;
            progress_cb(p, &format!("Zpracovávám řádek {} / {}", idx + 1, total_rows));
        }
    }

    progress_cb(0.95, "Ukládám soubor...");
    let mut finalni_seznam: Vec<Partner> = existujici_db.into_values().collect();
    finalni_seznam.sort_by(|a, b| a.id.cmp(&b.id));

    let nova_db = Databaze {
        posledni_aktualizace: aktualni_cas,
        zaznamy: finalni_seznam,
    };

    let file = File::create(json_path).map_err(|e| format!("Chyba při vytváření souboru: {}", e))?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &nova_db).map_err(|e| format!("Chyba při zápisu JSON: {}", e))?;

    Ok((format!("Import hotov.\nNové: {}\nZměněné: {}", pocet_novych, pocet_zmenenych), 0))
}

// --- VEŘEJNÉ FUNKCE ---

pub fn handle_database_update() -> Option<(String, i32)> {
    let excel_path = FileDialog::new()
        .add_filter("Excel", &["xlsx", "xlsm"])
        .set_title("Vyberte soubor")
        .pick_file()?;

    run_import_logic(excel_path, |_, _| {}).ok()
}

pub fn handle_database_update_with_progress<F>(excel_path: PathBuf, mut progress: F) -> Option<(String, i32)> 
where F: FnMut(f32, &str) + Send + 'static 
{
    run_import_logic(excel_path, |p, msg| progress(p, msg)).ok()
}