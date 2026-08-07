# Meetily Pro — Kanban khởi tạo

> **Snapshot:** 2026-08-07 • Đây là board khởi tạo sau khảo sát repo, không phải trạng thái implementation.
> Điểm là relative effort: **1** nhỏ, **2** nhỏ-vừa, **3** vừa, **5** lớn, **8** nên tách trước sprint planning.

## Quy ước board

### Vai trò

- **P** — Product/design
- **R** — Rust/Tauri/audio
- **F** — Frontend
- **M** — ML/audio evaluation
- **S** — Security/privacy/legal liaison
- **Q** — QA/release

Một card có một **DRI**; reviewer không thay DRI. “Legal” trong owner không có nghĩa kỹ sư tự đưa tư vấn pháp lý — cần người có thẩm quyền ký gate.

### Definition of Ready

Card chỉ vào `Ready` khi có: user outcome, owner, dependencies rõ, mock/API/data contract tối thiểu, test/acceptance condition, và không còn câu hỏi policy chặn implementation.

### Definition of Done

Card chỉ vào `Done` khi có: code review, test automation phù hợp, manual smoke trên platform liên quan, migration/rollback nếu đụng data, telemetry không chứa content, docs/release note nếu thay đổi hành vi người dùng. Với audio/ML phải có benchmark artifact; với privacy phải có review evidence.

### WIP limit đề xuất

| Cột | Limit | Lý do |
|---|---:|---|
| Ready | 6 | Không chuẩn bị quá nhiều trước khi M0 chốt. |
| In progress | 4 tổng / 1 DRI | Audio migration có blast radius lớn; ưu tiên hoàn tất. |
| Review | 3 | Review nhanh hơn mở thêm work. |
| Blocked | Không limit, review mỗi daily | Blocker phải có owner và ngày kiểm tra lại. |

---

## Board hiện tại

### Done — khảo sát/plan

| ID | Card | Evidence / output |
|---|---|---|
| `DISC-01` | Map kiến trúc supported app | Tauri/Next/Rust, audio pipeline, DB, templates và command surface được map trong [README](./README.md) + [Handoff](./HANDOFF.md). |
| `DISC-02` | Gap analysis Pro | Không thấy active implementation calendar, auto-join, chat/RAG, audit, TTS, Sherpa hay diarization end-to-end; VAD Silero đang ở pipeline. |
| `DISC-03` | Sherpa source/license reconnaissance | Fork URL, Apache-2.0, commit pin và sidecar proposal đã ghi lại; chưa thêm dependency. |

### Ready — M0 (bắt đầu theo thứ tự)

| ID | Card | DRI | Pts | Depends on | Acceptance condition |
|---|---|---:|---:|---|---|
| `PRO-001` | ADR: MIT Core vs Pro proprietary, entitlement và offline behavior | P + S | 3 | — | Quyết định A/B được owner phê duyệt; data cũ vẫn read/export được khi entitlement không có. |
| `PRO-002` | Product/legal policy: session modes, recording notice, cloud route, voice boundary | P + S | 5 | — | Consent language, retention default, auto-join boundary và voice-clone prohibited cases được ký. |
| `PRO-003` | Corpus rights manifest + accuracy benchmark protocol | M + Q | 5 | — | Corpus không nằm trong Git; quyền dùng, demographic/domain slices, normalization và metrics được review. |
| `PRO-004` | Threat model + secrets/privacy remediation ADR | S + R | 5 | — | Có inventory API keys/paths, plan OS keychain, scope Tauri FS và privacy-policy claim gap. |
| `PRO-005` | Sherpa sidecar technical/license spike | R + M | 5 | — | Pin/version, bridge API, target builds, model licenses, binary size/latency risks và rollback plan documented. |
| `PRO-006` | Session/provenance/audit data-model ADR | R + F | 5 | `PRO-001`, `PRO-002` | Additive schema, migration/rollback, API contracts và no-reuse rule cho speaker/voice embedding approved. |

### In progress

**Trống có chủ đích.** Chỉ kéo một card M0 sau khi xác nhận DRI/capacity. Không bắt đầu Sherpa integration, OAuth hay voice clone trước `PRO-001`–`PRO-006`.

### Review

**Trống.** M0 outputs cần product + security review chung, không review tuần tự tách rời.

### Blocked / policy gates

| ID | Card | Blocker | Next check |
|---|---|---|---|
| `PRO-041` | User-confirmed join/open-link automation | Platform ToS, recording-law UX và product decision từ `PRO-002` | Sau M0 legal review. |
| `PRO-053` | Voice clone enrollment/profile security | Consent policy, supported Vietnamese model license, threat model và delete/revocation design | Sau M6 controls + model evaluation. |
| `PRO-054` | Voice Labs launch gate | Red-team, legal/security sign-off, synthetic-label/watermark capability | Chỉ sau `PRO-050`–`PRO-053`. |

---

## Ordered product backlog

### M1 — Session foundation & trust

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-007` | Additive session schema + backward migration từ `meetings` | R | 5 | `PRO-006` | Existing meeting opens/edits/deletes without data loss; new session type/versioned metadata works. |
| `PRO-008` | Processing run/provenance + model manifest schema | R + M | 5 | `PRO-006` | Every new ASR/summary run records engine/model/config/source/timestamps; immutable result history. |
| `PRO-009` | OS credential store abstraction và migrate API keys khỏi SQLite | R + S | 8 | `PRO-004` | Keychain/Credential Manager/Secret Service adapters pass; SQLite secret remnants removed after verified migration. |
| `PRO-010` | Consent receipt + minimal audit event foundation | R + S + F | 5 | `PRO-002`, `PRO-006` | Recording/cloud-route consent is versioned, visible and recorded without content payload. |
| `PRO-011` | Session mode creation/import UI and metadata editor | F + R | 3 | `PRO-007` | Meeting, online class, Dharma talk work on create/import and old meeting defaults to meeting. |
| `PRO-012` | Mode template schema/validator + bundled Vietnamese templates | R + P | 3 | `PRO-006` | Dharma quote blocks require segment citation; template has version and locale. |
| `PRO-013` | Local glossary and reviewed correction history | F + R + M | 5 | `PRO-007`, `PRO-008` | Raw text survives, suggested term corrections are reviewable/reversible and scoped to profile/workspace. |

### M2 — Accuracy, model profiles and Sherpa VAD

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-014` | Reproducible ASR/VAD evaluation harness | M + Q | 5 | `PRO-003`, `PRO-008` | WER/CER, term accuracy, latency, RTF and dropped-segment report runs from manifest. |
| `PRO-015` | Model registry/downloader checksum/license manifest | R + M | 5 | `PRO-008` | Model SHA-256, source, license, target capability and deprecation state are enforced before use. |
| `PRO-016` | `AnalysisAudioBus`: aligned mic/system fan-out and bounded queues | R | 8 | `PRO-007` | Existing mixed playback remains compatible; analysis tracks align on shared clock and survive stress test. |
| `PRO-017` | `VoiceActivityProvider` interface + current Silero adapter | R | 5 | `PRO-016` | Current behavior is behind interface with unit/integration regression coverage. |
| `PRO-018` | `sherpa-bridge` POC and supervisor | R + M | 8 | `PRO-005`, `PRO-015`, `PRO-017` | Local IPC, health/restart, no TCP, model load/unload, failure fallback to Silero demonstrated on macOS/Windows. |
| `PRO-019` | Sherpa-vs-Silero benchmark and rollout decision | M + Q | 3 | `PRO-014`, `PRO-018` | Signed report decides opt-in/default/stop; no subjective promotion. |
| `PRO-020` | Smart silence/auto-pause UX with pre-roll | F + R | 5 | `PRO-017` | Opt-in only, visible state, no auto-stop, quiet-speech/long-pause tests pass. |
| `PRO-021` | Accuracy profiles + non-destructive retranscription | F + R + M | 5 | `PRO-008`, `PRO-015` | User selects live/postprocess profile; old run stays reproducible and comparable. |
| `PRO-022` | Audio failure, reconnect and sidecar crash test suite | Q + R | 5 | `PRO-016`–`PRO-021` | No silent transcript loss in scripted shutdown/reconnect/crash scenarios; failure is visible. |

### M3 — Template studio and export

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-023` | Template studio CRUD/version/preview | F + R | 5 | `PRO-012` | Create/duplicate/archive/import/export templates with JSON Schema validation and fixture preview. |
| `PRO-024` | Canonical `DocumentModel` and provenance mapper | R + F | 5 | `PRO-007`, `PRO-008`, `PRO-012` | UI and export consume same typed model; quote/summary/note semantics preserved. |
| `PRO-025` | Markdown renderer | R + Q | 3 | `PRO-024` | Stable, readable Markdown with citations/timecodes and golden fixtures. |
| `PRO-026` | DOCX renderer | R + Q | 5 | `PRO-024` | Styles, headings, tables/lists and Unicode/Vietnamese visual smoke tests pass. |
| `PRO-027` | PDF renderer POC + renderer selection ADR | R + Q | 5 | `PRO-024` | Cross-platform quality/size/accessibility trade-off documented; no unsupported claim. |
| `PRO-028` | Export commands/wizard/audit records | F + R | 5 | `PRO-025`–`PRO-027`, `PRO-010` | Destination picker, scope toggle, atomic write, no network egress, export event without content. |
| `PRO-029` | Golden export/accessibility/regression suite | Q | 5 | `PRO-028` | Semantic + visual snapshots for three modes and all formats pass on supported platforms. |

### M4 — Diarization and grounded chat

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-030` | Speaker-turn schema/migration with anonymous labels | R | 5 | `PRO-007`, `PRO-016` | New speaker tables/IDs, overlap/confidence/provenance; existing `transcripts.speaker` remains backward-compatible. |
| `PRO-031` | Diarization provider/sidecar POC + corpus report | R + M | 8 | `PRO-015`, `PRO-016`, `PRO-030` | DER/overlap report; no real-name identification; model failure degrades safely. |
| `PRO-032` | Timeline speaker relabel/merge/split UX | F + R | 5 | `PRO-030`, `PRO-031` | Manual corrections persist, override model, are undoable and appear in export. |
| `PRO-033` | Local retrieval index and invalidation worker | R + M | 5 | `PRO-008` | Index scoped per session by default; edit/delete/reprocess invalidates correct chunks. |
| `PRO-034` | Grounded chat command/provider routing | R + F | 5 | `PRO-009`, `PRO-033` | Local/external route explicit; prompt carries retrieved evidence rather than whole vault by default. |
| `PRO-035` | Citation/no-evidence guardrail and adversarial tests | M + Q | 5 | `PRO-034` | Factual response requires valid segment/time citation; unsupported question returns no-evidence. |
| `PRO-036` | Chat UX: scope, citations, feedback, correction | F | 5 | `PRO-034`, `PRO-035` | Citation opens transcript/audio; user sees provider/data route and can report error. |
| `PRO-037` | Knowledge-index privacy/retention integration | R + S | 3 | `PRO-010`, `PRO-033` | Delete/retention removes embeddings/cache; audit has no embedding or transcript content. |

### M5 — Calendar and safe automation

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-038` | `.ics` import/local calendar read-only integration | R + F | 3 | `PRO-007` | Event title/time can prefill session without external credential. |
| `PRO-039` | OAuth connector, least scopes, keychain token/revoke | R + S + F | 8 | `PRO-009`, `PRO-002` | Google/Microsoft connection/revocation tested; no token in DB/log/export. |
| `PRO-040` | Local detector/reminder/title-template suggestion | R + F | 5 | `PRO-011`, `PRO-038` | Opt-in detector stores no capture content before user starts; notification opens explicit choice. |
| `PRO-041` | Confirmed open/join link and one-click record flow | F + R | 3 | `PRO-002`, `PRO-040` | User confirmation required; no credential automation/unattended bot behavior. |
| `PRO-042` | Automation privacy/ToS/platform test matrix | S + Q | 3 | `PRO-039`–`PRO-041` | Provider/platform limitations and recording notice behavior signed off. |

### M6 — Compliance readiness and GA operations

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-043` | Retention, export/delete and recovery workflows | R + F + Q | 8 | `PRO-007`, `PRO-010`, `PRO-037` | Idempotent deletion covers DB, files, index/cache and has recovery/test evidence. |
| `PRO-044` | Tamper-evident audit log verifier | R + S | 5 | `PRO-010` | Verification detects mutation/gaps; audit never records raw content. |
| `PRO-045` | At-rest protection implementation and key lifecycle | R + S | 8 | `PRO-004`, `PRO-009` | Threat-model-selected encryption/key architecture passes migration/backup/recovery tests. |
| `PRO-046` | DPIA/ROPA/privacy policy/subprocessor documentation | S + P | 5 | `PRO-002`, `PRO-039`, `PRO-043`–`PRO-045` | Claims match shipping code; counsel/owner review recorded. |
| `PRO-047` | SBOM, NOTICE, model-license and vulnerability process | Q + R | 3 | `PRO-005`, `PRO-015` | Release artifact contains auditable third-party/model inventory. |
| `PRO-048` | Pen test, accessibility, DR and beta soak | Q + S + R | 8 | `PRO-022`, `PRO-029`, `PRO-035`, `PRO-042`–`PRO-047` | No open P0/P1; regression/recovery reports pass. |
| `PRO-049` | GA runbook, support playbook and release gate | P + Q | 3 | `PRO-046`–`PRO-048` | Owner checklist, known limits, rollback/update playbook and support escalation ready. |

### M7 — TTS and voice Labs

| ID | Card | DRI | Pts | Depends on | Done means |
|---|---|---:|---:|---|---|
| `PRO-050` | Local generic-TTS model/quality/license POC | R + M | 5 | `PRO-005`, `PRO-015` | Vietnamese/language quality, latency, bundle size, license and failure fallback report accepted. |
| `PRO-051` | TTS accessibility UX and synthetic provenance | F + R + Q | 5 | `PRO-050`, `PRO-024` | Playback/export is opt-in, clearly synthetic and linked to source text/run. |
| `PRO-052` | Voice-clone policy, consent and enrollment UX | P + S + F | 5 | `PRO-002`, `PRO-010`, `PRO-046` | Verified owner consent, scope, expiry, revoke/delete and prohibited-use flow are signed. |
| `PRO-053` | Isolated clone profile storage/synthesis security | R + S + M | 8 | `PRO-045`, `PRO-050`, `PRO-052` | Separate profile from diarization data; encrypted lifecycle, no automatic recording enrollment, abuse controls. |
| `PRO-054` | Labs red-team, revocation/delete proof and launch gate | Q + S + P | 5 | `PRO-051`–`PRO-053` | Impersonation, revoke, export labels and model capability checks pass; launch decision documented. |

---

## Sprint planning rules

1. Do not place more than one audio-pipeline structural change (`PRO-016`, `PRO-017`, `PRO-018`) in a sprint without its regression test card.
2. Do not combine a destructive DB migration with rendering, calendar or voice features. Migrations are additive, backed up and reversible at the app layer.
3. `PRO-009` precedes any calendar OAuth or new external provider credential.
4. `PRO-030` precedes `PRO-031`; no attempt to bolt diarization labels into the legacy `speaker` text field.
5. `PRO-035` must be complete before external beta of meeting chat.
6. `PRO-052`–`PRO-054` remain blocked unless legal/security explicitly remove the gate; “model works” is insufficient.

## Cadence

- **Daily:** review blocked cards, VAD/ASR regressions and any content-routing event.
- **Weekly:** product + security triage of policy changes, model licenses and customer corpus requests.
- **Sprint review:** demo using rights-cleared fixture for each of the three modes; present benchmark delta rather than only UI.
- **Release gate:** signed checklist from `PRO-049`; update `README`, privacy policy, supported matrix and known limitations before public claim.
