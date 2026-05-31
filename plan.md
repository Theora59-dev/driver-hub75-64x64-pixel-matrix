# Plan — Driver HUB75 pour écran 64×64 pixels (ESP32, Rust)

## 1. Contexte technique : protocole HUB75

Une matrice 64×64 utilise un **scan 1/32** : les 64 lignes sont divisées en 2 moitiés
de 32 lignes (haut et bas), adressées simultanément par le même jeu d'adresses
`A,B,C,D,E` (2^5 = 32 combinaisons). Le tramage d'une frame complète se fait ligne
par ligne, en activant une paire de lignes (une haute, une basse) à la fois.

**Signaux :**
| Signal | Rôle |
|--------|------|
| `R1,G1,B1` | Données RGB de la moitié haute (ligne `row_addr`) |
| `R2,G2,B2` | Données RGB de la moitié basse (ligne `row_addr + 32`) |
| `A,B,C,D,E` | Adresse de la paire de lignes (0–31) |
| `CLK` | Front montant = shifter le bit suivant dans les registres internes |
| `LAT` | Pulse de latch : transfère le registre de shift vers le registre de sortie |
| `OE` | Output Enable : `LOW` = allume les LEDs, `HIGH` = éteint |

**Séquence temporelle d'une paire de lignes :**
```
1. OE = HIGH   (éteindre les LEDs)
2. Adresser la ligne (A-E)
3. 64 pulses CLK en shifant R1,G1,B1,R2,G2,B2
4. LAT = pulse (monter/descendre)
5. OE = LOW    (allumer les LEDs pour la durée PWM)
6. Attendre la durée du bit-plane en cours
7. Passer à la paire de lignes suivante
```

Une frame complète = 32 paires de lignes × (1 sweep par bit-plane).

---

## 2. Architecture du driver

```
┌─────────────────────────────────────────────┐
│  Application / démo / embedded-graphics     │
├─────────────────────────────────────────────┤
│  Framebuffer (PixelMap<64, 64>)             │
│    ├─ DoubleBuffer (front / back)           │
│    └─ Conversion RGB565/RGB888 → bit-planes │
├─────────────────────────────────────────────┤
│  BCM Sequencer                              │
│    ├─ Découpe la frame en N bit-planes      │
│    └─ Ordonnance : [sweep₀, sweep₁, …]      │
├─────────────────────────────────────────────┤
│  Hub75 Output                               │
│    ├─ shift_row() → CLK + data sur GPIOs    │
│    ├─ latch()   → pulse LAT                 │
│    ├─ set_oe()  → PWM via timer/RMT         │
│    └─ set_row() → broches A–E               │
├─────────────────────────────────────────────┤
│  esp-hal (GPIO, Timer, RMT, I2S, DMA)       │
└─────────────────────────────────────────────┘
```

---

## 3. Modules détaillés

### 3.1 — `src/lib.rs`
Réexport public de l'API du driver. Contient le point d'entrée no_std.
Déclare les modules : `hub75`, `framebuffer`, `sequencer`.

### 3.2 — `src/framebuffer/`

**`color.rs`** — Types de couleurs :
- `Rgb565` (u16 : 5R|6G|5B)
- `Rgb888` (u8, u8, u8)
- Trait `Into<Rgb565>` pour accepter plusieurs formats
- Conversion Rgb888 → Rgb565 par masquage de bits

**`mod.rs`** — `PixelMap<const W: usize, const H: usize>` :
- Stockage plat `[Rgb565; W * H]`
- `write(x, y, color)` avec clipping
- `read(x, y) -> Rgb565`
- `clear(color)`
- Indexation : `y * W + x`

**Double buffering :**
- `DoubleBuffer` contient 2 `PixelMap` (`front`, `back`)
- `swap()` : échange atomique des pointeurs (ou `core::mem::swap`)
- `back_buffer()` pour dessiner
- `front_buffer()` pour le sequencer (lecture seule quand il lit pour shifter)
- Optionnel : `flush()` qui attend la fin du draw en cours puis swap

**Intégration embedded-graphics :**
- `DrawTarget` → transmet les appels au back buffer
- `OriginTopLeft`, dimensions 64×64

### 3.3 — `src/hub75/`

**`mod.rs`** — Types et fonctions bas niveau :
- `Hub75Pins` : structure contenant les `Output` GPIO pour chaque signal
  - `r1, g1, b1, r2, g2, b2` : 6 sorties data
  - `a, b, c, d, e` : 5 sorties adresse ligne
  - `clk, lat, oe` : 3 sorties contrôle
- Constructeur `new(gpio_pins...)` qui prend les `Output` déjà configurés
  (externalisation de la config pour flexibilité)
  - Variante : prendre directement les `Peripherals` + broches et tout configurer en interne

**Méthodes :**
- `set_clk(high: bool)`
- `set_data(r1,g1,b1,r2,g2,b2)` : met à jour les 6 bits data
- `pulse_clk()` : set_clk(true) + set_clk(false) (avec éventuel delay CPU)
- `latch()` : pulse sur LAT (monter, attendre, descendre)
- `set_oe(enabled: bool)` : true = éteint, false = allumé
- `set_row(addr: u8)` : positionne A–E selon les bits 0..4 de `addr`
- `shift_pixel(r1,g1,b1,r2,g2,b2)` : set_data + pulse_clk
- `shift_row(slice: &[PixelPair; 64])` : 64 appels à shift_pixel

**`PixelPair`** : (r1,g1,b1, r2,g2,b2) pour un même index de colonne.

**`timing.rs`** — Configuration matérielle pour le refresh :
- Init d'un `esp_hal::timer::Timer` (par ex. TimerGroup) pour l'IT de sweeping
- Ou init d'un canal `RMT` pour générer le train de pulses CLK
- Ou init de l'`I2S` en mode parallèle (LCD mode) pour shifter par DMA

### 3.4 — `src/sequencer/`

**`mod.rs`** — `Sequencer` : moteur principal du refresh.
- État : `current_row: u8`, `current_bitplane: u8`, `frame_in_progress: bool`
- `start()` : arme le timer (ou RMT) qui déclenche le handler périodique
- `stop()` : désarme le timer
- `set_framebuffer(&DoubleBuffer)` : donne l'accès au front buffer à lire
- Handler d'IT (timer ou RMT done) :
  1. Si début de frame : notifier le swap si nécessaire
  2. OE = HIGH
  3. Lire les 64 pixels du front buffer pour la ligne `current_row`
  4. Extraire le bit `current_bitplane` de chaque pixel pour R1,G1,B1,R2,G2,B2
  5. Shifter les 64 pixels via `Hub75Pins::shift_row()`
  6. Pulse LAT
  7. OE = LOW
  8. Attendre le délai PWM correspondant à `2^current_bitplane` unités
     *(le timer est reconfiguré pour le prochain tick dans cette attente)*
  9. `current_row++`
  10. Si `current_row == 32` : `current_row = 0`, `current_bitplane++`
  11. Si `current_bitplane == N` : frame terminée, `current_bitplane = 0`

**Important** : l'ISR doit être le plus court possible. Les 64 shifts peuvent être
fait directement ou via DMA/RMT selon la phase d'optimisation.

**`bcm.rs`** — Configuration du Binary Code Modulation :
- `BitDepth` enum : `Bpp1`, `Bpp2`, … `Bpp8`
- Table de conversion : niveau 0..(2^N - 1) → bits à afficher par sweep
- `gamma_correct(luminosity: u8, gamma: f32) -> u8` : lookup table précalculée
- Durées OE : `[1, 2, 4, 8, 16, 32, 64, 128]` × unité de temps de base
  (l'unité de base est calibrée pour atteindre ~60 FPS)

### 3.5 — `src/bin/main.rs`
- Init ESP32 : horloge, périphériques, GPIO
- Instanciation du framebuffer + double buffer
- Instanciation des `Hub75Pins`
- Instanciation du `Sequencer`
- Boucle principale : dessiner dans le back buffer, `swap()`, attendre

---

## 4. Détail par phase

### Phase 1 — Structure du projet

**Objectif :** Mettre en place l'architecture modulaire et les types de base.

**Étapes :**
1. Créer les dossiers `src/hub75/`, `src/framebuffer/`, `src/sequencer/`
2. Ajouter les `mod` dans `lib.rs`
3. Définir `Rgb565`, `Rgb888` dans `framebuffer/color.rs`
4. Définir `Hub75Pins` (squelette sans implémentation) dans `hub75/mod.rs`
5. Définir le trait `DisplayDriver { fn write_pixel(x, y, color); fn flush(); }`
   dans `lib.rs` (ou un module `driver`)
6. Compiler et vérifier (lint, taille binaire)

**Criteres de succès :** `cargo build` passe, la structure de dossiers est propre.

---

### Phase 2 — GPIO bit-bang : affichage statique

**Objectif :** Driver bas niveau qui shifte des motifs fixes à l'écran, sans
niveaux de gris (1 bit par couleur = 8 couleurs max).

**Étapes :**
1. Configurer les GPIOs en sortie dans `main.rs` :
   - `Output::new(pin, Level::Low, OutputConfig::default())` pour chaque signal.
2. Implémenter `Hub75Pins::set_row()`, `set_clk()`, `set_data()`, `pulse_clk()`,
   `latch()`, `set_oe()`.
3. Implémenter `Hub75Pins::shift_row(pixels)` :
   - Pour chaque colonne 0..64, appeler `pulse_clk()` après avoir placé les bits.
4. Dans `main.rs`, boucle de test :
   - Remplir un tableau `[PixelPair; 64]` avec un motif (ex: checkerboard).
   - Boucler sur les 32 paires de lignes :
     - `set_row(i)`
     - `shift_row(pixels)`
     - `latch()`
     - `set_oe(false)` → attendre 1ms → `set_oe(true)`
   - Refaire en boucle.
5. Vérifier à l'oscillo ou à l'œil que le motif s'affiche.

**Bits de la paire de lignes :**
Une `PixelPair` pour la colonne `c` contient les bits de la ligne `row` (haut)
et `row + 32` (bas) pour cette colonne :
```
(r1, g1, b1) = pixel(row, c)
(r2, g2, b2) = pixel(row + 32, c)
```
Pour l'instant 1 bit par couleur : un seuil à 128 pour convertir Rgb565 en
présence/absence de la couleur.

**Pièges :**
- Les GPIO ESP32 changent d'état sur écriture mémoire, pas besoin de delay.
- `CLK` doit être suffisamment lent pour que le registre à décalage du panneau
  ait le temps de lire. Tester à ~1–4 MHz. Un `nop` ou un petit `cortexm::asm::delay`
  après chaque `pulse_clk()` si nécessaire.
- Le pulse LAT doit être > 1 période CLK.

---

### Phase 3 — Framebuffer et double buffer

**Objectif :** Dessiner dans un buffer mémoire sans affecter l'affichage en cours.

**Étapes :**
1. `PixelMap<64, 64>` : stockage `[Rgb565; 4096]`, méthodes `write()`, `read()`.
2. `DoubleBuffer` : deux PixelMap, `swap()` atomique.
   - Protéger le swap avec `critical_section` ou un flag atomique.
3. Optionnel : implémenter `DrawTarget` from `embedded-graphics` (permet
   d'utiliser `Line`, `Circle`, `Text`, etc.).
4. Relier au sequencer : le sequencer lit depuis `front_buffer()`.

**Remarque :** Le swap doit être cohérent avec le refresh. Le pattern classique :
- Le sequencer signale "frame done" via un flag atomique `frame_ready`.
- `main()` dessine, puis attend `frame_ready` pour `swap()`.
- Ou l'inverse : le sequencer swap auto au début de la frame suivante.

---

### Phase 4 — Binary Code Modulation (BCM)

**Objectif :** Afficher des niveaux de gris (plusieurs bits par couleur) via
des sweeps multiples avec OE pondéré.

**Étapes :**
1. Définir `BitDepth` (ex: `Bpp6` = 6 bits/couleur, 64 niveaux, 6 sweeps/frame).
2. Modifier `shift_row` pour prendre un paramètre `bitplane: u8` :
   - Pour chaque colonne, extraire le bit `bitplane` des composantes R,G,B.
   - Shifter ce triplet de bits (1 si le bit est à 1, 0 sinon).
3. Modifier la boucle de balayage :
   - Pour `bitplane` de 0 à `N-1` :
     - Pour `row` de 0 à 31 :
       - OE off, adresser row, shifter le bit-plane, latch,
         OE on pour `2^bitplane` unités, OE off
4. Choisir l'unité de temps OE en fonction de la fréquence CLK :
   - Exemple : CLK à 2 MHz, 1 unité = 8 cycles CLK = 4 µs.
   - Sweep MSB (bit 5) : OE = 32 × 4µs = 128 µs par ligne.
   - Total des OE par frame = Σ(2^b) × 32 × 4µs = 63 × 128 µs = 8.064 ms.
   - Temps de shift par sweep : 64 × 32 × (1/CLK) = 2048 × 0.5 µs = 1.024 ms.
   - Total / frame = 6 × (1.024 + 8.064/6) ≈ 6 × 2.368 = 14.2 ms → ~70 FPS.
   - Ajuster l'unité de base pour viser 60 FPS (~16.6 ms).
5. Ajouter une table de correction gamma (256 entrées) appliquée au moment
   de l'écriture dans le framebuffer.

**Défi temps réel :** L'ISR ne peut pas faire des `delay_ms`. Il faut soit :
- **Option A : Timer one-shot.** Reconfigurer le timer à chaque étape pour
  le délai suivant.
- **Option B : Compteur matériel.** Démarrer un timer matériel en comptage,
  l'ISR suivante est déclenchée par la comparaison.
- **Option C : Boucle active courte.** Pour les petits bit-planes (0, 1, 2),
  une boucle `nop` dans l'ISR peut suffire si elle reste < ~10 µs.

Pour la première version, option A (timer one-shot) est plus propre :
1. Chaque handler termine en reprogrammant le timer pour le prochain événement.
2. Machine à états dans l'ISR :
   - `State::OeOff` → set_row, shift, latch, OE on, timer = 2^b unités.
   - `State::OeOn` → OE off, row++, passer à la ligne suivante ou au bit-plane suivant.

---

### Phase 5 — Séquenceur avec timer matériel

**Objectif :** Un timer `esp-hal` pilote le rafraîchissement en arrière-plan.

**Étapes :**
1. Initialiser un `Timer` (ex: `TimerGroup::new(peripherals.TIMG0).timer0`)
   avec un prescaler adapté pour des ticks de ~1 µs.
2. Configurer l'IT de comparaison (`set_interrupt_handler`) :
   - Handler qui appelle `Sequencer::tick()`.
3. L'état du sequencer est une machine à états :
   ```rust
   enum ScanState {
       LoadRow,   // OE off, shift, latch, start OE timer
       WaitOe,    // OE on, en attendant que le timer expire
   }
   ```
4. `tick()` selon l'état :
   - `LoadRow` :
     - `oe.set_high()`
     - `set_row(current_row)`
     - Lire les 64 pixels du front buffer, extraire le bit-plane
     - `shift_row()`
     - `latch()`
     - `oe.set_low()`
     - Recharger le timer avec `duration = 2^current_bitplane × base_unit`
     - Passer à `WaitOe`
   - `WaitOe` :
     - `current_row += 1`
     - Si `current_row == 32` : `current_row = 0`, `current_bitplane += 1`
     - Si `current_bitplane == bit_depth` : frame finie, reset
     - Repasser à `LoadRow`
5. La boucle principale (`main`) peut être complètement libre.
   - Elle écrit dans le back buffer.
   - Elle peut `swap()` quand `frame_done` flag est positionné.
   - Le sequencer lit `front_buffer` sans lock (lecture seule).

**Précautions :**
- L'ISR doit être courte. Éviter les divisions, modulo, boucles longues.
- Les accès au framebuffer doivent être rapides : tableau statique, pas de
  bounds check en release (ou déroulage manuel).
- En DEBUG, les performances seront bien moindres à cause du manque d'optimisation.
  Tester en `--release`.

---

### Phase 6 — RMT / I2S parallèle (optimisation)

**Objectif :** Remplacer le bit-bang dans l'ISR par un périphérique matériel
pour libérer le CPU.

**Option 6a — RMT (Remote Control) :**
- Le RMT peut générer un train de pulses préprogrammé (level + duration).
- Programme RMT pour 64 pixels :
  - 64 × (set_data + CLK high + CLK low) + LAT pulse + OE pulse.
  - Chaque étape = un `RmtSymbol` (level, 1 tick; level, duration).
- Avantage : l'ISR n'a plus qu'à lancer le DMA RMT et attendre la fin.
- Problème : la mémoire RMT est limitée (64 × 32 bits par canal sur ESP32).
  Pour 64 pixels, il faut 64 × 2 = 128 pulses pour data+CLK, plus latch et OE.
  Cela peut tenir en 1 ou 2 canaux. À vérifier dans la datasheet.

**Option 6b — I2S parallèle (LCD mode) :**
- L'I2S peut être configuré en mode parallèle 8 bits.
- Les 6 bits data + CLK (généré auto) + 1 bit inutilisé = 8 bits.
- Le DMA envoie un buffer de 64 octets (un par pixel) à vitesse configurable.
- L'ISR ne fait que changer la ligne et recharger le buffer DMA.
- C'est l'approche de la lib C++ `ESP32-HUB75-MatrixPanel-DMA`.
- Contrainte : nécessite que les GPIO data soient contigus (ou mappables).
  Sur ESP32, les bits I2S parallèle sont mappés à des GPIOs spécifiques.

**Stratégie recommandée :**
1. Faire la Phase 5 (timer + bit-bang) d'abord — ça marche pour des petites
   résolutions et bit depths.
2. Si les FPS sont insuffisants (> 16.6 ms par frame), implémenter l'option
   I2S parallèle (6b) qui est le standard industriel pour ce use case.

---

### Phase 7 — Tests et démos

**Motifs de test :**
- `0x0000, 0xFFFF` alternés → checkerboard
- Ramping horizontal : gradient R sur toute la ligne
- Barres verticales RGB (chaque colonne = couleur pure)
- Dégradé de gris sur toute la surface
- Animation : balle rebondissante (simple : position x,y qui se déplace)
- Scrolling text if `embedded-graphics` est intégré

**Mesures :**
- FPS = `frames_compté / durée` via un timer matériel
- `esp_println::println!()` toutes les secondes
- Impact CPU : mesurer le temps passé dans l'ISR vs loop principal

---

### Phase 8 — Polish

- Documentation `///` sur tous les items publics
- Exemples dans `examples/` :
  - `minimal.rs` : affichage d'un motif fixe
  - `animation.rs` : démo avec swap et mouvement
  - `gamma.rs` : comparaison gamma vs linéaire
- Cargo features : `bit-depth-4`, `bit-depth-6`, `embedded-graphics`
- Paramétrage : rendre le nombre de lignes, colonnes, et bit-depth configurables
  via const generics : `MatrixPanel<W, H, const BPP: u8>`
- Publier sur crates.io si souhaité

---

## 5. Brochage détaillé

Le brochage exact dépend de l'ESP32 et du panneau. Voici une proposition pour
un ESP32 DevKit C et un panneau HUB75 64×64 standard.

| HUB75 | GPIO | Notes |
|-------|------|-------|
| R1    | 4    | Data bit 0 |
| G1    | 5    | Data bit 1 |
| B1    | 6    | Data bit 2 |
| R2    | 7    | Data bit 3 |
| G2    | 8    | Data bit 4 |
| B2    | 9    | Data bit 5 |
| A     | 10   | Adresse LSB |
| B     | 11   | Adresse    |
| C     | 12   | Adresse    |
| D     | 13   | Adresse    |
| E     | 14   | Adresse MSB (64×64 nécessite 5 bits) |
| CLK   | 15   | Horloge série |
| LAT   | 16   | Latch |
| OE    | 17   | Output enable (NPN, LOW = allumé) |

Le brochage doit être facilement modifiable (passer les GPIO en paramètres).

---

## 6. Contraintes matérielles ESP32

- **Fréquence CPU** : 240 MHz max. Configurer avec `CpuClock::max()`.
- **Timers** : 2 TimerGroups × 2 timers = 4 timers hardware. 1 suffit pour
  le séquenceur.
- **GPIO** : Éviter GPIO 6–11 si la flash SPI est en mode DIO/QIO (ces pins
  sont utilisées par la flash). Sur la plupart des devkits, GPIO 6–11 sont
  les broches de la flash interne et ne doivent PAS être utilisées.
  - **Correction** : Ne pas utiliser GPIO 6–11. Revoir le brochage ci-dessus
    pour les déplacer vers des GPIO libres (par ex. GPIO 18–23, 25–27, 32–39).
  - GPIO 34–39 sont **input only** → ne pas les utiliser pour les sorties.
- **RMT** : 8 canaux, mémoire 64 × 32 bits par canal.
- **I2S parallèle** : Certains GPIOs sont fixes pour les bits parallèles :
  bit 0..7 → GPIO 4..11 (mais 6–11 posent problème avec la flash). Solution
  possible : utiliser le "I2S parallel" avec remapping via GPIO matrix.
- **Mémoire** : IRAM pour les ISR (pas de cache miss). Mettre les handlers
  en `#[ram]` si nécessaire.

### Brochage révisé (évite GPIO 6–11 et 34–39)

| HUB75 | GPIO |
|-------|------|
| R1    | 4    |
| G1    | 16   |
| B1    | 17   |
| R2    | 18   |
| G2    | 19   |
| B2    | 21   |
| A     | 22   |
| B     | 23   |
| C     | 25   |
| D     | 26   |
| E     | 27   |
| CLK   | 32   |
| LAT   | 33   |
| OE    | 15   |

---

## 7. Dépendances Cargo

```toml
[dependencies]
esp-hal = { version = "~1.1.0", features = ["esp32"] }
esp-bootloader-esp-idf = { version = "0.5.0", features = ["esp32"] }
esp-println = { version = "0.17.0", features = ["esp32"] }
critical-section = "1.2.0"

# Optionnel :
embedded-graphics = "0.8"       # DrawTarget, primitives
heapless = "0.8"                # Vec statique si nécessaire
micromath = "2.0"               # sqrt, sin, cos pour animations
```

---

## 8. Récapitulatif des FPS attendus

| Bit depth | Sweeps | CLK (MHz) | Shift (µs) | OE total (µs) | Frame (µs) | FPS  |
|-----------|--------|-----------|------------|---------------|------------|------|
| 1         | 1      | 4         | 512        | 32 × 1        | ~544       | 1838 |
| 2         | 2      | 4         | 1024       | 32 × 3        | ~1120      | 892  |
| 4         | 4      | 4         | 2048       | 32 × 15       | ~2528      | 395  |
| 6         | 6      | 4         | 3072       | 32 × 63       | ~5088      | 196  |
| 8         | 8      | 4         | 4096       | 32 × 255      | ~12256     | 81   |

*L'unité de base OE est 1 cycle CLK dans ce tableau. En pratique, il faudra
augmenter cette unité pour éviter un ratio cyclique trop faible aux petits
bit-planes, ce qui réduira les FPS. Les valeurs ci-dessus sont le maximum
théorique.*

---

## 9. Glossaire

| Terme | Définition |
|-------|------------|
| HUB75 | Standard de connecteur 16 pins pour matrices RGB LED |
| Scan 1/32 | 32 lignes adressées simultanément (2 moitiés de 32) |
| BCM   | Binary Code Modulation : chaque bit est un sweep avec OE pondéré |
| Sweep | Parcours complet des 32 paires de lignes pour un bit-plane donné |
| Bit-plane | Couche binaire d'une frame : tous les bits de poids `b` |
| Latch | Bascule le registre à décalage vers le registre de sortie |
| OE    | Output Enable, commande l'extinction/allumage global des LEDs |
| RMT   | Remote Control, périphérique ESP32 pour générer des pulses |
