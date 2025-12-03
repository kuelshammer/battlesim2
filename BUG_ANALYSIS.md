# Kampf-Simulations-Bugs Analyse (Kaelen2 Battle Log)

## ✅ Enhanced Logging funktioniert exzellent!

Die neuen Logs mit Bane/Advantage/Disadvantage Modifikatoren sind **perfekt**.
Beispiel: `Attack vs **Boris**: **15** vs AC 18 (-1d4 Bane, DISADVANTAGE) -> ❌ **MISS**`

## 🚨 4 massive entdeckte Fehler

### 1. "Zombie-Bane" Bug (Konzentration bricht nicht)
**Problem:** Bane-Effekt bleibt aktiv, obwohl Zauberin Erica tot ist.

**Analyse:**
- **Runde 1:** Erica wirft Bane, wird dann von Alestair getötet (`Erica falls unconscious`)
- **Runde 1 (später):** Van hat immer noch `-1d4 Bane` gegen Boris
- **Runde 2-3:** Andreas hat immer noch `-1d4 Bane` gegen Kaelen

**Ursache:** Das Konzentrationssystem bricht nicht automatisch, wenn der Zauberer stirbt.
**D&D Regel:** Bei 0 HP stirbt Konzentration sofort - alle Effekte enden.

**Root Cause im Code:** In `cleanup.rs` existiert `remove_dead_buffs()` aber der `on_unconscious` Event-Handler wird nicht korrekt getriggert.

### 2. Ranger "Cheat" (Doppelte Bonus-Aktion & Loop)
**Problem:** Andreas verwendet mehr Bonus-Aktionen als erlaubt.

**Analyse:**
- Jede Runde: `Hunter's Mark` (Bonus Aktion) + `Crossbow Expert` (Bonus Aktion)
- **Regel-Verstoß:** Ein Charakter hat nur **EINE** Bonus-Aktion pro Runde
- **AI-Fehler:** Hunter's Mark wird jede Runde neu auf dasselbe Ziel (Kaelen) gewirkt
  - Verschwendet unnötig Spell Slots
  - Hunter's Mark hält 1 Stunde, nicht 1 Runde

**Konfigurationsfehler in Kaelen2.json:**
```json
// FALSCH (Action-Slots vertauscht):
"Hunter's Mark" - actionSlot: 0 (sollte 1 sein - Bonus Action)
"Crossbow Expert" - actionSlot: 1 (sollte 0 sein - Action)
```

### 3. "Phantom HP Loss" bei Kaelen (HP Desync)
**Problem:** Kaelen verliert HP ohne dass ein Treffer im Log steht.

**Analyse:**
- **Start:** Kaelen (HP: 90/90)
- **Runde 1:** Kaelen wird NICHT getroffen (Andreas verfehlt mit 9 vs AC 15)
- **Runde 2 Start:** Kaelen (HP: 76/90) - **Wo sind 14 HP hin?**

**Mögliche Ursachen:**
- **A. "Cleave Bug":** Schaden von Van's Attack auf Boris (der getötet wird) wird fälschlicherweise auf Kaelen übertragen
- **B. Template Bug:** Hunter's Mark-Applikation verursacht unerwarteten Schaden
- **C. Logging Bug:** Schaden wird verursacht aber nicht korrekt geloggt

**Debug-Ansatz:** Verfolge alle Schadensberechnungen bei HP-Änderungen von Kaelen in Runde 1.

### 4. "Leichenschändung" (Multi-Attack auf Tote)
**Problem:** Charaktere greifen tote Ziele an.

**Analyse:**
- **Runde 1:** Van vs Boris
  - Attack 1: Treffer (10 Schaden) - Boris lebt noch ✓
  - Attack 2: Treffer (14 Schaden) - `Boris falls unconscious`
  - Attack 3: Noch ein Angriff → `skipping attack` (sollte vorher abgebrochen werden)

**Problem:** Der dritte Angriff wird gestartet, obwohl das Ziel nach dem zweiten Schlag bereits tot ist.
**Lösung:** Multi-Attack Sequenz sollte bei Ziel-Tod sofort unterbrochen werden, nicht erst vor dem nächsten Schlag prüfen.

---

## 🎯 Priorisierte Fix-Reihenfolge

### Phase 1: Kritisch (Breaking Simulation Reality)
1. **Fix Ranger Action Economy** - Korrigiere actionSlot in Kaelen2.json
2. **Fix Bane Concentration** - Implementiere on_unconscious cleanup

### Phase 2: Simulation Accuracy
3. **Debug Kaelen HP Loss** - Finde Phantom-Schadenquelle
4. **Fix Multi-Attack Logic** - Verhindere Angriffe auf tote Ziele

---

## 🔧 Technische Hinweise für Code-Änderungen

### Bane Fix
- Event-Handler in `resolution.rs` bei `falls unconscious`
- Trigger `cleanup::remove_dead_buffs()` mit Zauberer-ID
- Logging: `Template Bane dispelled (caster unconscious)`

### Ranger Fix
- Kaelen2.json actionSlots korrigieren
- Hunter's Mark template prüft: `if target.has_template("Hunter's Mark") -> skip casting`

### HP Loss Debug
- Log alle HP-Änderungen mit Ursprungsquelle
- Prüfe damage-applier mapping bei multi-attack

### Multi-Attack Fix
- Break-Condition zwischen einzelnen Attacken in `resolution.rs`
- Check: `if target.current_hp <= 0 { break; }`

**Die Analyse zeigt, dass das Logging-System perfekt funktioniert, aber die darunterliegende Kampfsimulation-Logik grundlegende D&D 5e Regeln verletzt.**