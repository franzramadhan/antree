# Antree: Game Design Document

Working title: Antree. Setting: Republik Kuvukiland.

## 1. Pitch

Antree is a 2D top-down satire game about getting one piece of paper from the government of Kuvukiland. The player is an ordinary citizen with a simple goal, a lost identity card or a birth certificate. The game turns that goal into a chain of offices, stamps, photocopies, and queues. Each office is a small desk minigame. Three meters track the cost: days, money, and conscience. The player finishes a run and says "yeah, that is exactly how it goes."

All in-game text is English, written in the register of a counter clerk: "Did you photocopy it?", "The system is down. Head office.", "Come back tomorrow.", "Anything for cigarettes?". Proper nouns use the Kuvukiland naming table in section 9.

## 2. Player fantasy

The player took a day off work to fix a document and wants to go home. The fantasy is survival with dignity: can you get through the system without becoming part of it? Every step offers a shortcut: a fixer, an inside man, grease money, cutting the line. Each one works and costs something. The player decides what kind of citizen they are, and the ending tells them, with no mercy and no praise.

## 3. The three meters

All numbers live in data files and are tunable.

| Meter | Range | Start | Direction |
|---|---|---|---|
| Days | 1 to quest deadline | 1 | Up is bad |
| Money | 0 to starting budget | quest budget | Down is bad |
| Conscience | 0 to 100 | 100 | Down is bad |

Currency is the Kuvuk, written Kv. The in-game day runs from 08:00 to 15:00 in 10 minute ticks (42 ticks). Queue numbers stop at 11:30. Travel between buildings costs 10 to 20 minutes. The photocopy stall stays open until 17:00. When the clock passes 15:00 or the player chooses "Go home", Days increases by 1 and the clock resets.

Conscience costs per shortcut:

| Action | Conscience | Money | Effect |
|---|---|---|---|
| Cut the line | -10 | 0 | Skip 3 to 6 places, 15 percent chance of being sent to the back |
| Pay the fixer | -15 | 50 to 150 Kv | Skip to the front of the queue |
| Grease money (negotiation) | -20 | 100 to 300 Kv | Skip all remaining obstacles at that office today |
| Call the inside man | -25 | 0 | The current counter issues at once. Once per run, after the first completed quest |
| Fill the form with nonsense | -5 | 0 | Instant form, 30 percent chance of a "data correction" loop later |
| Confront the line cutter | 0 | 0 | 50 percent the queue is restored, 20 percent a fight and a lost day |

Conscience never increases during a run. Refusing a bribe costs nothing but adds weight to the obstacle roll (see section 6).

Grade math. Each quest defines par_days, official_fees, and budget.

```
day_score        = clamp(100 - 12 * max(0, days - par_days), 0, 100)
extra_spent      = max(0, (budget - money) - official_fees)
money_score      = clamp(100 - 100 * extra_spent / (budget - official_fees), 0, 100)
conscience_score = conscience
total            = round(0.4 * day_score + 0.3 * money_score + 0.3 * conscience_score)
```

A run fails when days exceeds the deadline or money reaches 0 with a required fee still unpaid.

## 4. Quests and document chains

Quests unlock in order. Finishing a quest with any grade except F unlocks the next. Each chain has one hidden requirement that the first office does not mention, and one wrong requirement that the information board lists but no office asks for. Both are picked by seed from a small pool per quest.

| # | Quest | Visible chain | Hidden requirement pool | Wrong requirement pool | Par days | Deadline | Budget | Official fees |
|---|---|---|---|---|---|---|---|---|
| 1 | Lost KTPSS | Poldur SK (loss report), Pak Gnilak (cover letter), Kuvukiland Small District Sabar Menanti (cover letter), Dukcatan (KTPSS) | KSK photocopy in triplicate, red background photo | SKTPA, 2 duty stamps | 4 | 8 | 400 Kv | 0 Kv |
| 2 | Get an SKTPA | Pak Gnilak, Small District, Poldur SK (SKTPA), BKFN (certified copies, 5) | 4x6 photo, health letter from Puskesnung | SPUSP, yellow folder | 5 | 10 | 500 Kv | 30 Kv |
| 3 | Renew the SIM | Puskesnung (health letter), Klinik Sabar (queue psychology test), Satpan (new SIM) | Old SIM expired one day ago, so the theory test is required | Small District cover letter | 3 | 7 | 600 Kv | 120 Kv |
| 4 | Birth certificate | Puskesnung (birth note), Pak Gnilak, Small District, Dukcatan (certificate) | Parent names must match the KTPSS exactly, one typo triggers the data correction loop | Marriage certificate certified by BKFN | 5 | 10 | 500 Kv | 0 Kv |
| 5 | Building permit (PBYM) | BPYMS (land letter), DTRYB (zoning recommendation), Small District (neighbour no-objection), DPSPTD (PBYM) | DPSPTD front door is locked, side door opens via the inside man or after 2 days of waiting; SPUSP needed before the cover letter | SKTPA, health letter | 8 | 16 | 1500 Kv | 250 Kv |
| 6 | Death certificate | Puskesnung (death note), Pak Gnilak, Small District, Dukcatan (certificate) | Proof from Pak Gnilak that the deceased was alive, KSK photocopy in triplicate | Red background photo, yellow folder | 5 | 10 | 500 Kv | 0 Kv |

Chain resolution: an office grants a document when the player holds its requirements. The quest ends when the goal document is in the inventory. Offices can be visited in any order.

## 5. Minigame catalog

Each office runs one minigame per visit, picked from office data. Failure costs time, never the run.

### 5.1 Form Filling
Form with 6 to 10 fields about the player's own data. Rules at the top set the date format, capitals, and dashes in numbers. Order, labels, and rules change per visit. Each field cycles through three values: correct, wrong format, typo. Timer 60 seconds. Submit with one wrong field: "Rejected. Fill it in again, sir.", 30 minutes lost. Three rejections: "Come back tomorrow.", visit ends. Option "Fill with nonsense": instant pass, Conscience -5, flagged for a later data correction.

### 5.2 Stamp Hunt
Office floor plan with 6 to 10 desks. Exactly one clerk holds the stamp. Other clerks give directions, 30 percent of them wrong. Each visited desk costs 10 minutes. The holder has a status rolled per visit: present (35 percent), on break (25 percent), in a meeting (20 percent), out on field duty (20 percent). Break and meeting end after 30 to 60 minutes. Field duty ends the visit. Failure: the clock passes 14:30 before the stamp lands.

### 5.3 Photocopy Run
The clerk lists requirements for 4 seconds: copy count, duty stamp value, folder colour, certified or not. The player runs to the BKFN photocopy stall across the road, picks items from a shelf, and runs back. Travel 15 minutes each way. Wrong item: the clerk rejects, run again. Failure: the office closes at 11:30 for new files. Money: each copy and duty stamp costs Kv.

### 5.4 The Queue
Queue of 6 to 14 people. Each tick one event may fire: an auntie cuts in with "just a quick question" (15 percent), the inside man walks past (8 percent), a fixer offers a slot (15 percent). Actions: stay quiet (lose the place), confront (roll per section 3), cut the line (Conscience -10), pay the fixer (Money, Conscience -15). The counter serves one number every 1 to 2 ticks, so a patient player is served in 10 to 30 ticks. Failure: the counter closes before your number.

### 5.5 System Down
A progress bar that stalls at random points. Actions: wait (10 minutes per press), refresh (asks the clerk, 3 refreshes in a row annoy them and add 20 minutes), go home (lose the number, come back tomorrow), ask Mas Reronoh (he knows the system comes back in N minutes, costs a cup of coffee, 15 Kv, no Conscience cost). Failure: the office closes.

### 5.6 Negotiation
Dialog tree. The clerk hints at an "unofficial administrative fee" without ever saying a number first. Options: pay (Money, Conscience -20, skip remaining obstacles here today), decline politely (no cost, obstacle roll gets +20 weight on bad outcomes), report it (10 percent: free pass and a clerk replaced, 90 percent: your file becomes "under review" for 2 days). A fixer cannot be reported. The game never comments on the choice.

### 5.7 Queue Theory Test
Quest 3 only. Ten multiple choice questions about queue etiquette, written as a real theory test with absurd content. Pass at 7 correct. Fail: retake tomorrow, 25 Kv per attempt.

## 6. Obstacle table

Rolled once on arrival at each counter, seeded. Weights are per office in data and can be overridden. Default weights:

| Obstacle | Weight | Effect |
|---|---|---|
| Smooth | 25 | Straight to the minigame |
| System down | 15 | System Down minigame first |
| Clerk on break | 15 | Wait 30 to 60 minutes |
| Missing paperwork | 15 | Photocopy Run first |
| Form format changed | 10 | Form Filling with a new layout even if already done |
| Fixer offers help | 10 | Negotiation, half the time with the clerk, half with a fixer at the door |
| Counter moved buildings | 5 | 20 minutes travel, roll again |
| Surprise collective leave | 5 | Office closed today, Days +1 |

Modifiers: after declining politely in Negotiation, add 20 to every non-Smooth weight at that office for the rest of the run. After grease money, force Smooth at that office for the day. After calling the inside man, halve every non-Smooth weight for the rest of the run.

## 7. Endings and grades

| Grade | Condition | Title | Closing line |
|---|---|---|---|
| S | total >= 85 and conscience >= 80 | Model Citizen | "You did it. Nobody believes you, but you did it." |
| A | total >= 70 | Patient Citizen | "Patience pays. Your wallet did not." |
| B | total >= 50 | Average Citizen | "Done. More or less. Less, mostly." |
| C | total >= 30 | Tired Citizen | "The document is finished. You are too." |
| D | total < 30 | Resigned Citizen | "What matters is it got done, sir. What matters is it got done." |
| X | complete, days <= par_days, conscience <= 40 | Part of the System | "Congratulations, you are now part of the system. The next queue number is your child's." |
| F | deadline passed or money 0 | Home Empty-Handed | "Come back tomorrow, sir. The day after is also fine." |

Grade X overrides S to D. The ending screen shows the three meters, the grade, a one-line "clerk's note" picked from a pool, the seed, and a restart button. Best grade per quest is saved.

Title screen disclaimer: "Kuvukiland is a fictional country. Any resemblance to any real country is purely coincidental, allegedly." README repeats it.

## 8. Art and audio direction

Internal canvas 640x360, integer scaled, letterboxed. Pixel art, 16x16 tiles. Palette: faded government beige, mint green walls, brown wood counters, one saturated red for stamps and "REJECTED". Original cozy top-down art drawn by tools/sprites.py: 16x24 characters with 4-way walk cycles, composed buildings, trees, and a 3x office interior with a blinking clerk. UI panels look like carbon copy forms with dotted lines. Font: Pixel Operator (CC0), 8 and 16 pixel sizes.

Audio: synthesized at boot. Stamp thud for approvals, buzz for refusals, dot matrix printer for forms, counter bell for queue advances and doors, a modem stutter for system down, a coin clink for Money, a click for other input. Music: "Please Hold", an original 2/4 hold tune that loops forever, synthesized from MIDI at boot. M mutes everything.

Juice: screen shake and a red "REJECTED" stamp on refusals, a stamp slam on approval and the title, cracks in the Conscience bar below 40.

Tutorial: form ORI-01 before the first errand. Each row explains one control. Blank rows are refused, and the complete form is refused once for the wrong ink. Tab on quest select replays it.

## 9. Kuvukiland naming table

| Role | Kuvukiland name |
|---|---|
| Country | Republik Kuvukiland |
| Capital | Kota Antree Raya |
| Village office | Kuvukiland Small District Sabar Menanti |
| Sub-district | Ketamacan Besok Lagi |
| District | Kabupaten Tunggu Sebentar |
| Neighbourhood head | Pos Ketua Lingkungan, Pak Gnilak |
| Civil registry | Dinas Kependudukan dan Pencatatan Antrian (Dukcatan) |
| One-stop licensing | Dinas Perizinan Satu Pintu Tapi Pintunya Dikunci (DPSPTD) |
| Police certificate desk | Kantor Polpolan Sektor Tidur (Poldur SK) |
| Licence issuing unit | Satuan Penyelenggara Antrian (Satpan) |
| National photocopy agency | Badan Koordinasi Fotokopi Nasional (BKFN) |
| Public clinic | Pusat Kesehatan Menunggu (Puskesnung) |
| Psych test clinic | Klinik Sabar |
| Land office | Badan Pertanahan Yang Masih Sengketa (BPYMS) |
| Spatial planning | Dinas Tata Ruang Yang Berubah (DTRYB) |
| Identity card | Kartu Tanda Penduduk Sementara Selamanya (KTPSS) |
| Family card | Kartu Susunan Keluarga (KSK) |
| Police clearance | Surat Keterangan Tidak Pernah Antree (SKTPA) |
| Driving licence | Surat Izin Mengantri (SIM) |
| Cover letter | Surat Pengantar Untuk Surat Pengantar (SPUSP) |
| Building permit | Persetujuan Bangunan Yang Mungkin (PBYM) |
| Birth certificate | Akta Kelahiran Yang Sah Katanya |
| Death certificate | Akta Kematian Menunggu Konfirmasi |
| Currency | Kuvuk (Kv) |
| Titles | Pah Harul, Bu Tamac, Pak Dibak, Mas Reronoh, Ibu Tekol 3, Pak Maptas |
| Archetypes | clerk, fixer, line-cutting auntie, inside man, security guard, angry uncle |

## 10. Cut list

1. Ambient fan audio loop (cut in Phase 5).
2. Quest 5 (PBYM). Longest chain, most new offices.
3. Queue Theory Test minigame (Quest 3 falls back to Form Filling).
4. Report option in Negotiation.
5. Touch input.
6. Stamp Hunt wrong directions (holder shown directly, status roll stays).
7. Quest 4 (birth certificate). Quests 2 and 3 cover the mechanics.
8. Photocopy Run memory element (requirements stay on screen).
9. Seed sharing UI (seed still shown on the ending screen).
10. Inside man contact unlock.

Phase 5 cut item 1, because a fan loop needs a recording. Touch stays optional.
