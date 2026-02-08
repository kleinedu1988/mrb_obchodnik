## [0.1.2] - 2024-05-23
### Přidáno
- **Moderní ProgressBar**: Implementace komponentu s modrým akcentem (`#4aa3df`).
- **ImportDialog**: Nový dialog v nastavení pro přehlednější workflow výběru listů z Excelu.
- **Parsování IČO**: Rozšířená podpora pro různé datové formáty (String, Int, Float).

### Změněno
- **Backend**: Kompletní refaktorizace synchronizační logiky (využití `Entry API` pro `HashMap` a `BufWriter` pro rychlejší zápis JSON).
- **UI**: Modernizace karet `StatusCard` a `FilterCard` s interaktivními hover efekty a lepším ohraničením.
- **API**: Sjednocení vlastností mezi `main.slint` a `settings.slint` pro stabilnější kompilaci.

### Opraveno
- **Kompilace**: Oprava kritických chyb při inferenci typů u `calamine` dat.
- **Linter**: Odstranění varování o nepoužívaných importech a opravy paddingu v Slintu.
- **UX**: Oprava chyby, kdy overlay nahrávání neblokoval vstupy v pozadí.

## [0.1.1] - 2024-05-22
### Změněno
- Interní reorganizace UI: rozdělení do složek `ui/components` a `ui/views`.
- Grafické prvky (Sidebar, Home, Import, Settings) přesunuty do samostatných souborů pro lepší přehlednost.
- Úprava rozměrů hlavního okna na 1300x850px pro lepší čitelnost.

### Opraveno
- Oprava vertikálního zarovnání lupy a vyhledávacího pole v Nastavení.
- Odstranění kolizí v bindingu vlastností u Sidebaru.
- Přidání vnitřních okrajů (padding) do jednotlivých stránek, aby obsah nelícoval s okraji okna.

## [0.1.0] - 2024-05-20
### Přidáno
- První základní funkční rozhraní aplikace se sidebar navigací.