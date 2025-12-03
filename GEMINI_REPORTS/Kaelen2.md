warning: unused variable: `attacker_bane_debuff`
   --> src/resolution.rs:252:17
    |
252 |             let attacker_bane_debuff: Vec<String> = attacker.final_state.buffs.values()
    |                 ^^^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_attacker_bane_debuff`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `damage_before_multiplier`
   --> src/resolution.rs:413:21
    |
413 |                 let damage_before_multiplier = damage;
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_damage_before_multiplier`

warning: variable does not need to be mutable
   --> src/resolution.rs:707:27
    |
707 |     for (is_target_enemy, mut target_idx) in raw_targets.iter().copied() {
    |                           ----^^^^^^^^^^
    |                           |
    |                           help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `a`
   --> src/simulation.rs:774:26
    |
774 |         Action::Template(a) => {
    |                          ^ help: if this is intentional, prefix it with an underscore: `_a`

warning: `simulation-wasm` (lib) generated 4 warnings (run `cargo fix --lib -p simulation-wasm` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running `target/debug/examples/run_sim_from_file ../Kaelen2.json`
Running simulation with 4 players and 1 encounters...
  [DEBUG] Checking pre-combat action for Kaelen: Rage (Slot: -3)
        Checking usability for Kaelen: Rage. Remaining uses: Some(4.0)
        Getting targets for Kaelen's action: Rage. Allies: 3, Enemies: 4
          Buff 1/1 of Kaelen. Attempting to select target.
            Selecting ally target (Strategy: Self_). Allies available: 3. Excluded: []
              Self target selected.
            Target selected for Kaelen: Ally Kaelen
        Kaelen found 1 total targets for action Rage.
  [DEBUG] Checking pre-combat action for Boris: Bless (Slot: -3)
        Checking usability for Boris: Bless. Remaining uses: Some(2.0)
        Getting targets for Boris's action: Bless. Allies: 3, Enemies: 4
          Template 1/1 of Boris. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Boris: Enemy Bob
        Boris found 1 total targets for action Bless.

--- Round START ---
  Turn Order: ["Team1 15.0", "Team1 14.0", "Team1 12.0", "Team2 10.0", "Team1 4.0", "Team2 3.0", "Team2 2.0"]
  Turn for: Andreas (Init: 15.0)
      Getting actions for Andreas. Creature actions: 3
        Considering action: Hunter's Mark (Slot: 1, Freq: Static("at will"))
        Checking usability for Andreas: Hunter's Mark. Remaining uses: None
          Action Hunter's Mark usable. Adding to result.
        Considering action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark (Slot: 0, Freq: Static("at will"))
        Checking usability for Andreas: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark. Remaining uses: None
          Action Hand Crossbow x2 + Crossbow Expert + Hunter's Mark usable. Adding to result.
        Considering action: Crossbow Expert (Slot: 0, Freq: Static("at will"))
          Slot 0 already used this turn.
      Chose action: Hunter's Mark
        Getting targets for Andreas's action: Hunter's Mark. Allies: 4, Enemies: 3
          Template 1/1 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Boris")
            Target selected for Andreas: Enemy Boris
        Andreas found 1 total targets for action Hunter's Mark.
      Selected 1 targets.
      Chose action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
        Getting targets for Andreas's action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark. Allies: 4, Enemies: 3
          Attack 1/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Boris")
            Target selected for Andreas: Enemy Boris
          Attack 2/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Boris")
            Target selected for Andreas: Enemy Boris
          Attack 3/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Boris")
            Target selected for Andreas: Enemy Boris
        Andreas found 3 total targets for action Hand Crossbow x2 + Crossbow Expert + Hunter's Mark.
      Selected 3 targets.
  Andreas turn END. Current State: P1 HP: 57.0, P2 HP: 90.0
  Turn for: Van (Init: 14.0)
      Getting actions for Van. Creature actions: 2
        Considering action: Action Surge: Greatsword x2 (Slot: 5, Freq: Static("1/fight"))
        Checking usability for Van: Action Surge: Greatsword x2. Remaining uses: Some(1.0)
          Action Action Surge: Greatsword x2 usable. Adding to result.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Van: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
      Chose action: Action Surge: Greatsword x2
        Getting targets for Van's action: Action Surge: Greatsword x2. Allies: 4, Enemies: 3
          Attack 1/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Boris")
            Target selected for Van: Enemy Boris
          Attack 2/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Boris")
            Target selected for Van: Enemy Boris
        Van found 2 total targets for action Action Surge: Greatsword x2.
      Selected 2 targets.
        [DEBUG] remove_all_buffs_from_source called: Source ID: 493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0
          Cleared concentration and action targets on dead caster: Boris
      Chose action: Greatsword x2
        Getting targets for Van's action: Greatsword x2. Allies: 4, Enemies: 3
          Attack 1/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Erica")
            Target selected for Van: Enemy Erica
          Attack 2/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Erica")
            Target selected for Van: Enemy Erica
        Van found 2 total targets for action Greatsword x2.
      Selected 2 targets.
        [DEBUG] remove_all_buffs_from_source called: Source ID: 8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0
          Cleared concentration and action targets on dead caster: Erica
  Van turn END. Current State: P1 HP: 57.0, P2 HP: 90.0
  Turn for: Bob (Init: 12.0)
      Getting actions for Bob. Creature actions: 1
        Considering action: Dodge (Slot: 0, Freq: Static("at will"))
        Checking usability for Bob: Dodge. Remaining uses: None
          Action Dodge usable. Adding to result.
      Chose action: Dodge
        Getting targets for Bob's action: Dodge. Allies: 4, Enemies: 3
          Buff 1/1 of Bob. Attempting to select target.
            Selecting ally target (Strategy: Self_). Allies available: 4. Excluded: []
              Self target selected.
            Target selected for Bob: Ally Bob
        Bob found 1 total targets for action Dodge.
      Selected 1 targets.
  Bob turn END. Current State: P1 HP: 57.0, P2 HP: 90.0
  Turn for: Boris (Init: 10.0)
    Boris is dead, skipping turn.
  Turn for: Alestair (Init: 4.0)
      Getting actions for Alestair. Creature actions: 3
        Considering action: Lay on Hands (Slot: 0, Freq: Static("1/day"))
        Checking usability for Alestair: Lay on Hands. Remaining uses: Some(1.0)
          Action Lay on Hands condition not met.
        Considering action: Divine Smite (Slot: 5, Freq: Limited { reset: "lr", uses: 3 })
        Checking usability for Alestair: Divine Smite. Remaining uses: Some(3.0)
          Action Divine Smite usable. Adding to result.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Alestair: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
      Chose action: Divine Smite
        Getting targets for Alestair's action: Divine Smite. Allies: 4, Enemies: 3
          Buff 1/1 of Alestair. Attempting to select target.
            Selecting ally target (Strategy: Self_). Allies available: 4. Excluded: []
              Self target selected.
            Target selected for Alestair: Ally Alestair
        Alestair found 1 total targets for action Divine Smite.
      Selected 1 targets.
      Chose action: Greatsword x2
        Getting targets for Alestair's action: Greatsword x2. Allies: 4, Enemies: 3
          Attack 1/2 of Alestair. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Alestair: Enemy Kaelen
          Attack 2/2 of Alestair. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Alestair: Enemy Kaelen
        Alestair found 2 total targets for action Greatsword x2.
      Selected 2 targets.
  Alestair turn END. Current State: P1 HP: 57.0, P2 HP: 55.0
  Turn for: Kaelen (Init: 3.0)
      Getting actions for Kaelen. Creature actions: 5
        Considering action: Rage (Slot: -3, Freq: Limited { reset: "lr", uses: 4 })
        Checking usability for Kaelen: Rage. Remaining uses: Some(3.0)
          Action Rage condition not met.
        Considering action: Reckless Attack (Slot: 5, Freq: Static("at will"))
        Checking usability for Kaelen: Reckless Attack. Remaining uses: None
          Action Reckless Attack usable. Adding to result.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Kaelen: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
        Considering action: Longtooth (Slot: 1, Freq: Static("at will"))
        Checking usability for Kaelen: Longtooth. Remaining uses: None
          Action Longtooth usable. Adding to result.
        Considering action: Death Damage (Slot: 6, Freq: Limited { reset: "lr", uses: 1 })
        Checking usability for Kaelen: Death Damage. Remaining uses: Some(1.0)
          Action Death Damage usable. Adding to result.
      Chose action: Death Damage
        Getting targets for Kaelen's action: Death Damage. Allies: 3, Enemies: 4
          Attack 1/1 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Kaelen: Enemy Bob
        Kaelen found 1 total targets for action Death Damage.
      Selected 1 targets.
      Chose action: Reckless Attack
        Getting targets for Kaelen's action: Reckless Attack. Allies: 3, Enemies: 4
          Buff 1/1 of Kaelen. Attempting to select target.
            Selecting ally target (Strategy: Self_). Allies available: 3. Excluded: []
              Self target selected.
            Target selected for Kaelen: Ally Kaelen
        Kaelen found 1 total targets for action Reckless Attack.
      Selected 1 targets.
      Chose action: Greatsword x2
        Getting targets for Kaelen's action: Greatsword x2. Allies: 3, Enemies: 4
          Attack 1/2 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Kaelen: Enemy Bob
          Attack 2/2 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Kaelen: Enemy Bob
        Kaelen found 2 total targets for action Greatsword x2.
      Selected 2 targets.
        [DEBUG] remove_all_buffs_from_source called: Source ID: 4371996e-e952-489a-abba-be9d495307d6-3-0
          Cleared concentration and action targets on dead caster: Bob
          Removed buff '854675ae-36eb-4e0c-917b-6b04a01a0220' from Bob (source 4371996e-e952-489a-abba-be9d495307d6-3-0 is dead)
          Bob had 1 buffs from dead source, now has 0 buffs total
      Chose action: Longtooth
        Getting targets for Kaelen's action: Longtooth. Allies: 3, Enemies: 4
          Attack 1/1 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Andreas")
            Target selected for Kaelen: Enemy Andreas
        Kaelen found 1 total targets for action Longtooth.
      Selected 1 targets.
  Kaelen turn END. Current State: P1 HP: 57.0, P2 HP: 55.0
  Turn for: Erica (Init: 2.0)
    Erica is dead, skipping turn.
CLEANUP: Removing buffs from dead sources: {"4371996e-e952-489a-abba-be9d495307d6-3-0", "493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0", "8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0"}
CLEANUP: Removing buffs from dead sources: {"4371996e-e952-489a-abba-be9d495307d6-3-0", "493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0", "8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0"}

--- Round START ---
  Turn Order: ["Team1 15.0", "Team1 14.0", "Team1 12.0", "Team2 10.0", "Team1 4.0", "Team2 3.0", "Team2 2.0"]
  Turn for: Andreas (Init: 15.0)
      Getting actions for Andreas. Creature actions: 3
        Considering action: Hunter's Mark (Slot: 1, Freq: Static("at will"))
        Checking usability for Andreas: Hunter's Mark. Remaining uses: None
          Action Hunter's Mark usable. Adding to result.
        Considering action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark (Slot: 0, Freq: Static("at will"))
        Checking usability for Andreas: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark. Remaining uses: None
          Action Hand Crossbow x2 + Crossbow Expert + Hunter's Mark usable. Adding to result.
        Considering action: Crossbow Expert (Slot: 0, Freq: Static("at will"))
          Slot 0 already used this turn.
      Chose action: Hunter's Mark
        Getting targets for Andreas's action: Hunter's Mark. Allies: 4, Enemies: 3
          Template 1/1 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
        Andreas found 1 total targets for action Hunter's Mark.
      Selected 1 targets.
      Chose action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
        Getting targets for Andreas's action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark. Allies: 4, Enemies: 3
          Attack 1/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
          Attack 2/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
          Attack 3/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
        Andreas found 3 total targets for action Hand Crossbow x2 + Crossbow Expert + Hunter's Mark.
      Selected 3 targets.
  Andreas turn END. Current State: P1 HP: 57.0, P2 HP: 23.0
  Turn for: Van (Init: 14.0)
      Getting actions for Van. Creature actions: 2
        Considering action: Action Surge: Greatsword x2 (Slot: 5, Freq: Static("1/fight"))
        Checking usability for Van: Action Surge: Greatsword x2. Remaining uses: Some(0.0)
          Action Action Surge: Greatsword x2 not usable.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Van: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
      Chose action: Greatsword x2
        Getting targets for Van's action: Greatsword x2. Allies: 4, Enemies: 3
          Attack 1/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Van: Enemy Kaelen
          Attack 2/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Van: Enemy Kaelen
        Van found 2 total targets for action Greatsword x2.
      Selected 2 targets.
  Van turn END. Current State: P1 HP: 57.0, P2 HP: 10.0
  Turn for: Bob (Init: 12.0)
    Bob is dead, skipping turn.
  Turn for: Boris (Init: 10.0)
    Boris is dead, skipping turn.
  Turn for: Alestair (Init: 4.0)
      Getting actions for Alestair. Creature actions: 3
        Considering action: Lay on Hands (Slot: 0, Freq: Static("1/day"))
        Checking usability for Alestair: Lay on Hands. Remaining uses: Some(1.0)
          Action Lay on Hands usable. Adding to result.
        Considering action: Divine Smite (Slot: 5, Freq: Limited { reset: "lr", uses: 3 })
        Checking usability for Alestair: Divine Smite. Remaining uses: Some(2.0)
          Action Divine Smite condition not met.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
          Slot 0 already used this turn.
      Chose action: Lay on Hands
        Getting targets for Alestair's action: Lay on Hands. Allies: 4, Enemies: 3
          Heal 1/1 of Alestair. Attempting to select target.
            Selecting injured ally target (Strategy: AllyWithLeastHP). Allies available: 4. Excluded: []
              Considering injured ally Andreas. HP: 45.0/54.0
              Considering injured ally Bob. HP: 0.0/20.0
            Selected injured ally target: Some("Bob")
            Target selected for Alestair: Ally Bob
        Alestair found 1 total targets for action Lay on Hands.
      Selected 1 targets.
  Alestair turn END. Current State: P1 HP: 57.0, P2 HP: 10.0
  Turn for: Kaelen (Init: 3.0)
      Getting actions for Kaelen. Creature actions: 5
        Considering action: Rage (Slot: -3, Freq: Limited { reset: "lr", uses: 4 })
        Checking usability for Kaelen: Rage. Remaining uses: Some(3.0)
          Action Rage condition not met.
        Considering action: Reckless Attack (Slot: 5, Freq: Static("at will"))
        Checking usability for Kaelen: Reckless Attack. Remaining uses: None
          Action Reckless Attack usable. Adding to result.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Kaelen: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
        Considering action: Longtooth (Slot: 1, Freq: Static("at will"))
        Checking usability for Kaelen: Longtooth. Remaining uses: None
          Action Longtooth usable. Adding to result.
        Considering action: Death Damage (Slot: 6, Freq: Limited { reset: "lr", uses: 1 })
        Checking usability for Kaelen: Death Damage. Remaining uses: Some(0.0)
          Action Death Damage not usable.
      Chose action: Reckless Attack
        Getting targets for Kaelen's action: Reckless Attack. Allies: 3, Enemies: 4
          Buff 1/1 of Kaelen. Attempting to select target.
            Selecting ally target (Strategy: Self_). Allies available: 3. Excluded: []
              Self target selected.
            Target selected for Kaelen: Ally Kaelen
        Kaelen found 1 total targets for action Reckless Attack.
      Selected 1 targets.
      Chose action: Greatsword x2
        Getting targets for Kaelen's action: Greatsword x2. Allies: 3, Enemies: 4
          Attack 1/2 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Kaelen: Enemy Bob
          Attack 2/2 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Kaelen: Enemy Bob
        Kaelen found 2 total targets for action Greatsword x2.
      Selected 2 targets.
      Chose action: Longtooth
        Getting targets for Kaelen's action: Longtooth. Allies: 3, Enemies: 4
          Attack 1/1 of Kaelen. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 4. Excluded: []
            Selected target: Some("Bob")
            Target selected for Kaelen: Enemy Bob
        Kaelen found 1 total targets for action Longtooth.
      Selected 1 targets.
        [DEBUG] remove_all_buffs_from_source called: Source ID: 4371996e-e952-489a-abba-be9d495307d6-3-0
          Cleared concentration and action targets on dead caster: Bob
  Kaelen turn END. Current State: P1 HP: 57.0, P2 HP: 10.0
  Turn for: Erica (Init: 2.0)
    Erica is dead, skipping turn.
CLEANUP: Removing buffs from dead sources: {"493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0", "4371996e-e952-489a-abba-be9d495307d6-3-0", "8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0"}
CLEANUP: Removing buffs from dead sources: {"493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0", "4371996e-e952-489a-abba-be9d495307d6-3-0", "8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0"}

--- Round START ---
  Turn Order: ["Team1 15.0", "Team1 14.0", "Team1 12.0", "Team2 10.0", "Team1 4.0", "Team2 3.0", "Team2 2.0"]
  Turn for: Andreas (Init: 15.0)
      Getting actions for Andreas. Creature actions: 3
        Considering action: Hunter's Mark (Slot: 1, Freq: Static("at will"))
        Checking usability for Andreas: Hunter's Mark. Remaining uses: None
          Action Hunter's Mark usable. Adding to result.
        Considering action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark (Slot: 0, Freq: Static("at will"))
        Checking usability for Andreas: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark. Remaining uses: None
          Action Hand Crossbow x2 + Crossbow Expert + Hunter's Mark usable. Adding to result.
        Considering action: Crossbow Expert (Slot: 0, Freq: Static("at will"))
          Slot 0 already used this turn.
      Chose action: Hunter's Mark
        Getting targets for Andreas's action: Hunter's Mark. Allies: 4, Enemies: 3
          Template 1/1 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
        Andreas found 1 total targets for action Hunter's Mark.
      Selected 1 targets.
      Chose action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
        Getting targets for Andreas's action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark. Allies: 4, Enemies: 3
          Attack 1/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
          Attack 2/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
          Attack 3/3 of Andreas. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Andreas: Enemy Kaelen
        Andreas found 3 total targets for action Hand Crossbow x2 + Crossbow Expert + Hunter's Mark.
      Selected 3 targets.
  Andreas turn END. Current State: P1 HP: 57.0, P2 HP: 1.0
  Turn for: Van (Init: 14.0)
      Getting actions for Van. Creature actions: 2
        Considering action: Action Surge: Greatsword x2 (Slot: 5, Freq: Static("1/fight"))
        Checking usability for Van: Action Surge: Greatsword x2. Remaining uses: Some(0.0)
          Action Action Surge: Greatsword x2 not usable.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Van: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
      Chose action: Greatsword x2
        Getting targets for Van's action: Greatsword x2. Allies: 4, Enemies: 3
          Attack 1/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Van: Enemy Kaelen
          Attack 2/2 of Van. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            Selected target: Some("Kaelen")
            Target selected for Van: Enemy Kaelen
        Van found 2 total targets for action Greatsword x2.
      Selected 2 targets.
        [DEBUG] remove_all_buffs_from_source called: Source ID: ee727462-cfe0-452a-bf82-9bdd101e05db-0-0
          Cleared concentration and action targets on dead caster: Kaelen
          Removed buff '9a0d2c76-cef3-472b-89c1-41433860f246' from Kaelen (source ee727462-cfe0-452a-bf82-9bdd101e05db-0-0 is dead)
          Removed buff '7a2f6115-ede5-452d-b38c-b4b0ff192485' from Kaelen (source ee727462-cfe0-452a-bf82-9bdd101e05db-0-0 is dead)
          Kaelen had 2 buffs from dead source, now has 0 buffs total
  Van turn END. Current State: P1 HP: 57.0, P2 HP: 0.0
  Turn for: Bob (Init: 12.0)
    Bob is dead, skipping turn.
  Turn for: Boris (Init: 10.0)
    Boris is dead, skipping turn.
  Turn for: Alestair (Init: 4.0)
      Getting actions for Alestair. Creature actions: 3
        Considering action: Lay on Hands (Slot: 0, Freq: Static("1/day"))
        Checking usability for Alestair: Lay on Hands. Remaining uses: Some(0.0)
          Action Lay on Hands not usable.
        Considering action: Divine Smite (Slot: 5, Freq: Limited { reset: "lr", uses: 3 })
        Checking usability for Alestair: Divine Smite. Remaining uses: Some(2.0)
          Action Divine Smite condition not met.
        Considering action: Greatsword x2 (Slot: 0, Freq: Static("at will"))
        Checking usability for Alestair: Greatsword x2. Remaining uses: None
          Action Greatsword x2 usable. Adding to result.
      Chose action: Greatsword x2
        Getting targets for Alestair's action: Greatsword x2. Allies: 4, Enemies: 3
          Attack 1/2 of Alestair. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            No target found for Alestair's attack 1.
          Attack 2/2 of Alestair. Attempting to select target.
            Selecting enemy target (Strategy: EnemyWithLeastHP). Enemies available: 3. Excluded: []
            No target found for Alestair's attack 2.
        Alestair found 0 total targets for action Greatsword x2.
      Selected 0 targets.
  Alestair turn END. Current State: P1 HP: 57.0, P2 HP: 0.0
  Turn for: Kaelen (Init: 3.0)
    Kaelen is dead, skipping turn.
  Turn for: Erica (Init: 2.0)
    Erica is dead, skipping turn.
CLEANUP: Removing buffs from dead sources: {"4371996e-e952-489a-abba-be9d495307d6-3-0", "8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0", "493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0", "ee727462-cfe0-452a-bf82-9bdd101e05db-0-0"}
CLEANUP: Removing buffs from dead sources: {"4371996e-e952-489a-abba-be9d495307d6-3-0", "8c731fce-c353-4cb9-b1fb-fed5e5ea16be-2-0", "493f9da5-92ec-45da-b5bd-4a04fa8452df-1-0", "ee727462-cfe0-452a-bf82-9bdd101e05db-0-0"}

=== SIMULATION LOGS ===

--- Encounter Start: Players vs Monsters ---

=== Pre-Combat Setup ===
  > Kaelen uses Rage
      -> Casts Rage on Kaelen
  > Boris uses Bless
    - Applying template: Bless
      Template Bless applied to target


# Round 1

## Andreas (HP: 54/54)
    - Uses Action: Hunter's Mark
    - Applying template: Hunter's Mark
      Template Hunter's Mark applied to target
    - Uses Action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
* ⚔️ Attack vs **Boris**: **21** vs AC 18 -> ✅ **HIT**
  * 🩸 Damage: **19**
* ⚔️ Attack vs **Boris**: **6** vs AC 18 (MISS!) -> ❌ **MISS**
* ⚔️ Attack vs **Boris**: **15** vs AC 18 -> ❌ **MISS**

## Van (HP: 57/57)
    - Uses Action: Action Surge: Greatsword x2
* ⚔️ Attack vs **Boris**: **13** vs AC 18 -> ❌ **MISS**
* ⚔️ Attack vs **Boris**: **23** vs AC 18 -> ✅ **HIT**
  * 🩸 Damage: **10**
  * 💀 **Boris falls unconscious!**
    - Uses Action: Greatsword x2
* ⚔️ Attack vs **Erica**: **22** vs AC 18 -> ✅ **HIT**
  * 🩸 Damage: **18**
* ⚔️ Attack vs **Erica**: **30** vs AC 18 (CRIT!) -> ✅ **HIT**
  * 🩸 Damage: **18**
  * 💀 **Erica falls unconscious!**

## Bob (HP: 20/20)
    - Uses Action: Dodge
      -> Casts Dodge on Bob

## Alestair (HP: 54/54)
    - Uses Action: Divine Smite
      -> Casts Divine Smite on Alestair
    - Uses Action: Greatsword x2
* ⚔️ Attack vs **Kaelen**: **16** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **43** (Base 28 + Buffs 15) * 0.50 [] = **21**
* ⚔️ Attack vs **Kaelen**: **17** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **29** (Base 23 + Buffs 6) * 0.50 [] = **14**

## Kaelen (HP: 55/90)
    - Uses Action: Death Damage
* ⚔️ Attack vs **Bob**: **120** vs AC 14 (CRIT!) -> ✅ **HIT**
  * 🩸 Damage: **11**
    - Uses Action: Reckless Attack
      -> Casts Reckless Attack on Kaelen
    - Uses Action: Greatsword x2
* ⚔️ Attack vs **Bob**: **15** vs AC 14 (ADVANTAGE) -> ✅ **HIT**
  * 🩸 Damage: **13**
  * 💀 **Bob falls unconscious!**
      -> Bob is already unconscious, skipping attack
    - Uses Action: Longtooth
* ⚔️ Attack vs **Andreas**: **28** vs AC 18 (ADVANTAGE) -> ✅ **HIT**
  * 🩸 Damage: **9**

# Round 2

## Andreas (HP: 45/54)
    - Uses Action: Hunter's Mark
    - Applying template: Hunter's Mark
      Template Hunter's Mark applied to target
    - Uses Action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
* ⚔️ Attack vs **Kaelen**: **23** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **21** * 0.50 [] = **10**
* ⚔️ Attack vs **Kaelen**: **25** vs AC 15 (CRIT!) -> ✅ **HIT**
  * 🩸 Damage: **25** * 0.50 [] = **12**
* ⚔️ Attack vs **Kaelen**: **22** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **21** * 0.50 [] = **10**

## Van (HP: 57/57)
    - Uses Action: Greatsword x2
* ⚔️ Attack vs **Kaelen**: **26** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **13** * 0.50 [] = **6**
* ⚔️ Attack vs **Kaelen**: **16** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **15** * 0.50 [] = **7**

## Alestair (HP: 54/54)
    - Uses Action: Lay on Hands
      -> Heals Bob for 35 HP (was at -15/20)

## Kaelen (HP: 10/90)
    - Uses Action: Reckless Attack
      -> Casts Reckless Attack on Kaelen
    - Uses Action: Greatsword x2
* ⚔️ Attack vs **Bob**: **14** vs AC 14 (ADVANTAGE) -> ✅ **HIT**
  * 🩸 Damage: **16**
* ⚔️ Attack vs **Bob**: **13** vs AC 14 (ADVANTAGE) -> ❌ **MISS**
    - Uses Action: Longtooth
* ⚔️ Attack vs **Bob**: **16** vs AC 14 (ADVANTAGE) -> ✅ **HIT**
  * 🩸 Damage: **7**
  * 💀 **Bob falls unconscious!**

# Round 3

## Andreas (HP: 45/54)
    - Uses Action: Hunter's Mark
    - Applying template: Hunter's Mark
      Template Hunter's Mark applied to target
    - Uses Action: Hand Crossbow x2 + Crossbow Expert + Hunter's Mark
* ⚔️ Attack vs **Kaelen**: **22** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **19** * 0.50 [] = **9**
* ⚔️ Attack vs **Kaelen**: **11** vs AC 15 -> ❌ **MISS**
* ⚔️ Attack vs **Kaelen**: **9** vs AC 15 -> ❌ **MISS**

## Van (HP: 57/57)
    - Uses Action: Greatsword x2
* ⚔️ Attack vs **Kaelen**: **24** vs AC 15 -> ✅ **HIT**
  * 🩸 Damage: **10** * 0.50 [] = **5**
  * 💀 **Kaelen falls unconscious!**
      -> Kaelen is already unconscious, skipping attack

## Alestair (HP: 54/54)
    - Uses Action: Greatsword x2
      -> No valid targets (skipping execution)

=== RESULTS ===

Encounter 1: 3 rounds
