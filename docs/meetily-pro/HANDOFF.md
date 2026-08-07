# Meetily Pro — Handoff kỹ thuật và vận hành

> **Prepared:** 2026-08-07 (UTC), kickoff updated 2026-08-08 (Asia/Colombo)
> **Branch:** `arena/019fdda3-meetily`
> **Repository baseline:** `0281737d87d26352fb0adc78c8c0975f691b23d1`
> **Planning commit:** `95dcd57` (`docs: add Meetily Pro roadmap and handoff`)
> **Kickoff implementation:** session-type vertical slice is now underway; see [DECISIONS](./DECISIONS.md) for the current delivered scope and the remaining gates.

## 1. Bàn giao nhanh cho người nhận việc

### Đọc theo thứ tự

1. [DECISIONS](./DECISIONS.md) — các lựa chọn product owner đã chốt.
2. [README](./README.md) — mục tiêu, repo findings, safety boundaries.
3. [MILESTONES](./MILESTONES.md) — thứ tự delivery/gates.
4. [KANBAN](./KANBAN.md) — card IDs, dependency, definition of done.
5. Phần này — file map, kiến trúc target, migration/API/test plan.

### Không được làm ngay

- Không thêm endpoint vào `backend/`; README backend xác nhận đó là archive không supported.
- Không thay Silero VAD bằng Sherpa trực tiếp trong `pipeline.rs` mà chưa có interface, benchmark và fallback.
- Không dùng `transcripts.speaker` để lưu danh tính/diarization mới. Migration hiện hữu mô tả nó là `mic`/`system`; speaker turns cần schema riêng.
- Không thêm Google/Microsoft OAuth token hay voice sample vào SQLite settings.
- Không ship hoặc bật voice clone chỉ vì TTS model chạy được.
- Không gọi build “đã pass”: sandbox tại thời điểm khảo sát có Node `v22.22.3`, nhưng không có `pnpm`, Rust/cargo hoặc dependency/build cache; chưa chạy test/build.

## 2. Bản đồ mã hiện tại

| Concern | File/path cần đọc | Điều cần biết trước khi sửa |
|---|---|---|
| Tauri entrypoint | `frontend/src-tauri/src/lib.rs` | Toàn bộ command registration, setup, tray, database startup; đây là nơi đăng ký command mới. |
| Bundle/permissions | `frontend/src-tauri/tauri.conf.json` | Có `externalBin` cho `llama-helper`, `ffmpeg`; capability FS hiện khá rộng (`fs:write-all`, `fs:read-all`). Review trước khi thêm sidecar/artifact. |
| Dependencies | `frontend/src-tauri/Cargo.toml` | Có `silero_rs`, `ort`, `whisper-rs`, `sqlx`; chưa có Sherpa. Tránh collision ONNX runtime bằng sidecar POC trước. |
| Audio capture/lifecycle | `audio/recording_manager.rs`, `audio/recording_commands.rs`, `audio/recording_saver.rs` | Hiện tạo mixed recording/checkpoints và transcript event. Giữ contract cũ trong refactor. |
| Live pipeline | `audio/pipeline.rs`, `audio/vad.rs` | Raw mic/system được ring-buffer trộn rồi VAD Silero; live VAD redemption 400 ms. Đây là lý do cần analysis fan-out trước mix. |
| ASR abstraction | `audio/transcription/{provider,engine,worker}.rs` | Có trait cho ASR và event `transcript-update`; chưa có trait VAD/diarization/provenance đầy đủ. |
| Batch import/reprocess | `audio/import.rs`, `audio/retranscription.rs`, `audio/common.rs` | Import long audio có VAD, cancellation, segments/timestamps; là path tốt để benchmark post-process. |
| Database | `database/manager.rs`, `database/models.rs`, `database/repositories/`, `migrations/` | SQLite + SQLx migrate. Existing migrations include `speaker`, licensing, notes; use additive migrations only. |
| Settings/secrets | `database/repositories/setting.rs` | API keys/custom OpenAI config đang được đọc/ghi ở SQLite. `PRO-009` phải giải quyết trước OAuth/cloud expansion. |
| Summary | `summary/`, `summary/templates/`, `summary/template_commands.rs` | Templates JSON bundled/custom/listed/validated, nhưng chưa có CRUD per-session/version persistence. |
| Frontend summary | `hooks/meeting-details/useSummaryGeneration.ts`, `components/MeetingDetails/*` | Summary payload đã giữ timestamp text; template chọn giữ trong state client, hiện default lại `standard_meeting`. |
| UI/session | `app/page.tsx`, `app/meeting-details/`, `components/Sidebar/` | Bắt đầu từ mode selector + metadata, nhưng giữ navigation/meeting ID để tương thích. |
| Policies | `PRIVACY_POLICY.md`, `README.md` | Privacy policy có `[Current Date]` và claim encryption-at-rest cần xác minh/điều chỉnh trước public compliance claim. |

## 3. Kiến trúc target — boundaries để tránh monolith mới

### 3.1 Đề xuất layout core (sau khi `PRO-001` chốt license)

Các interface/core có thể ở repo này. Nếu chọn Pro proprietary, implementation thương mại nằm trong private crate/repo và được bundle trong app Pro, không commit vào MIT core.

```text
frontend/src-tauri/src/
  session/
    commands.rs          # create/update/list session metadata
    models.rs            # SessionType, SessionMetadata, ProcessingRun
    repository.rs
  processing/
    run.rs               # orchestration/state machine/provenance
    model_registry.rs
    quality.rs
  audio/
    analysis_bus.rs      # NEW: aligned mic/system fan-out, bounded queues
    vad_provider.rs      # NEW: VoiceActivityProvider trait + Silero adapter
    diarization.rs       # interface only; implementation optional
    sherpa_supervisor.rs # bridge lifecycle; ideally feature-gated
  documents/
    model.rs             # canonical DocumentModel, citation types
    markdown.rs
    docx.rs
    pdf.rs
    commands.rs
  knowledge/
    index.rs
    retrieval.rs
    chat.rs
    citations.rs
  integrations/
    calendar.rs
    detector.rs
  privacy/
    consent.rs
    audit.rs
    retention.rs
    secrets.rs
  voice/
    tts.rs
    profile.rs           # only if Labs approval passed
```

Do **not** make one large `pro.rs`. Keep runtime interfaces small, serializable and versioned at Tauri boundary. Pro implementations may implement traits behind capability checks; core behavior should remain usable if an entitlement is absent.

### 3.2 Session state machine

```text
created
  -> capturing | imported
  -> processing (one or more ProcessingRun)
  -> ready
  -> exported (optional)
  -> retained | delete_pending -> deleted
```

A session may have multiple runs: live transcript, high-accuracy retranscript, diarization, summary, index and export. Never overwrite the only transcript in place. `active_run_id` is a user-visible selection; prior runs are retained/deleted by policy.

### 3.3 Content semantics

Use a generic `Session` in new UI/API while preserving existing `meetings` storage/API compatibility.

- `meeting`: decisions/action items can exist.
- `online_class`: learning objectives, concepts, Q&A, exercises.
- `dharma_talk`: `exact_quote`, `summary`, `editorial_note`, `practice_note`; only `exact_quote` represents a transcript span and requires citation.

A generated source/citation must store `segment_id`, `audio_start_time`, `audio_end_time`, `processing_run_id`; do not manufacture canonical scripture references or teacher attributions.

## 4. Data model and migration plan

### 4.1 Additive schema proposal

Names are proposals; final names/constraints belong in `PRO-006` ADR.

| Table / change | Minimum fields | Purpose |
|---|---|---|
| `meetings` additive fields or `sessions` compatibility view | `session_type`, `metadata_json`, `schema_version`, `active_processing_run_id`, `consent_id` | Preserve existing IDs while adding mode and typed metadata. |
| `processing_runs` | `id`, `meeting_id`, `kind`, `status`, `provider`, `model_id`, `model_sha256`, `config_json`, `input_artifact_ref`, `started_at`, `finished_at`, `metrics_json`, `parent_run_id` | Reproducibility, comparison and rollback. |
| `transcript_revisions` or additive transcript run fields | `id`, `processing_run_id`, `raw_text`, `normalized_text`, time range, confidence, source_track, sequence, `supersedes_id` | Keep raw ASR and reviewed normalization apart. Do not bulk rewrite legacy rows first. |
| `speaker_labels` | `id`, `meeting_id`, `display_name`, `kind=anonymous|manual`, `created_by`, timestamps | Human-readable label only; no biometric identity by default. |
| `speaker_turns` | `id`, `meeting_id`, `processing_run_id`, `speaker_label_id`, start/end, confidence, overlap, `source_track` | Diarization result/provenance. |
| `template_versions` | `id`, `template_id`, semantic version, schema/content JSON, locale, created/archived timestamps | Pin summary output to a reproducible template. |
| `glossary_entries` + `glossary_changes` | canonical/variant/locale/type/source/review state | Pāli/Sanskrit/Hán–Việt/domain corrections with approval history. |
| `document_exports` | `id`, session/run/template refs, format, destination class, checksum, created_at | Export provenance; never store destination path in a central audit if privacy policy disallows it. |
| `consent_receipts` | policy version, scope, route, timestamp, revocation state | Recording/cloud/voice consent. |
| `audit_events` | UUID, actor pseudonym, action, entity ref, policy version, timestamp, `prev_hash`, `event_hash` | Tamper-evident log without transcript/audio/secret payload. |
| `calendar_connections` | provider, account pseudonym, scopes, token **secure reference only**, sync status | OAuth metadata; token is in OS credential store. |
| knowledge index manifest | session/run/chunk/model refs, state, delete marker | Vectors/index data stored in protected local artifact, not raw payload in audit. |
| `voice_profiles` (Labs only) | random ID, consent ref, encrypted artifact ref, model/version, expiry/revocation | Separate from speaker embedding/diarization entirely. |

### 4.2 Migration rules

1. Create migration with a version greater than all current SQLx migrations (currently through `20251229000000`). Use a new `202608...` prefix.
2. Take/offer backup before first structural migration; test a copied v0.4.0 DB in CI fixture.
3. Add columns/tables/indexes first. Read old rows with defaults; write dual-compatible data until upgrade path is proven.
4. Do **not** `DROP` a customer table or re-create tables in a migration for these features. Existing licensing migration has destructive history; do not repeat that pattern.
5. Backfill in resumable batches and record a migration checkpoint. App must continue if the device is low on disk or a migration is interrupted.
6. Verify delete cascade for transcript, processing artifacts, embeddings, speaker turns, export metadata and voice profiles independently.
7. Do not call a DB rollback “safe” unless tested on a backup. App-level compatibility/backup is preferred for SQLite irreversible schema changes.

### 4.3 Existing `speaker` caveat

`20251110000001_add_speaker_field.sql` says values are `mic` or `system`, but current active model/API/UI paths do not expose a complete speaker-diarization feature. Treat it as **audio source provenance**, not a person. New `speaker_turns` links to a label and retains source track separately.

## 5. Tauri API contracts to design before UI coding

Exact Rust naming can follow project conventions; payloads should be typed and versioned.

```ts
// Session and processing
api_create_session({ title, sessionType, metadata, consentId })
api_update_session_metadata({ sessionId, patch, schemaVersion })
api_list_processing_runs({ sessionId })
api_start_processing_run({ sessionId, profileId, sourceTracks, parentRunId? })
api_activate_processing_run({ sessionId, runId })

// Templates and artifacts
api_create_template_version({ templateId?, content, locale })
api_preview_template({ templateVersionId, fixtureOrSessionId })
api_export_session({ sessionId, runId, templateVersionId, format, scopes })

// Speakers and knowledge
api_list_speaker_turns({ sessionId, runId })
api_apply_speaker_edits({ sessionId, baseRevision, operations })
api_build_knowledge_index({ sessionId, runId })
api_ask_session({ sessionId, scope, question, providerRoute })

// Privacy/integrations
api_get_data_route({ providerRoute })
api_record_consent({ scope, policyVersion, choice })
api_apply_retention({ policyId, dryRun? })
api_verify_audit_log({ from?, to? })
api_calendar_connect({ provider })
api_calendar_disconnect({ connectionId })

// Voice Labs only
api_tts_synthesize({ textRef, voiceMode: 'generic' | 'profile', profileId? })
api_create_voice_profile({ consentReceiptId, enrollmentArtifactRef })
api_revoke_voice_profile({ profileId })
```

Validation requirements:

- Every ID is constrained to the calling local workspace/user context before filesystem/provider use.
- Commands return user-safe error codes; diagnostics never include API keys, transcript text or raw audio.
- Long work emits progress/cancellation events like current import/retranscribe paths.
- Chat returns `answer`, `answer_state`, `citations[]`, `provider_route`, `processing_run_id`; client rejects factual answer with an empty citation list.

## 6. Sherpa implementation handoff

### 6.1 Source facts captured for the spike

- Requested fork: `https://github.com/Pabhassaracitto/sherpa-onnx`
- Pin observed on 2026-08-07: `6897144f087712d0972648fb9ece6ca211b5ee41`
- Project documents local VAD, TTS, diarization and Rust/C/C++ APIs; repository license is Apache-2.0.
- Models have independent licenses/terms; model license review is not covered by Apache-2.0 runtime license.

### 6.2 Recommended POC sequence

1. Create an internal `sherpa-bridge` proof-of-concept outside the critical live pipeline. It receives a versioned binary frame: session/run ID, 16 kHz mono samples, source track and timestamp.
2. Build/use Sherpa through a supported API and pin source/release + build flags. Capture SHA-256 of bridge binary and every model artifact.
3. Package per target through Tauri external binary mechanism after verifying naming/signing requirements for each platform; do not download executables at runtime.
4. Launch child with inherited minimum environment; use stdin/stdout length-delimited protocol or platform local pipe. No localhost listener, no network port.
5. Implement protocol: `hello`, `load_model`, `segment`, `flush`, `health`, `unload`, `shutdown`, `error`; all have protocol version and max message size.
6. Add `SherpaSupervisor`: child lifecycle, bounded request queue, timeout, one retry/restart, fallback event to Silero, local metrics only.
7. Implement `VoiceActivityProvider` and write a `SileroVadProvider` adapter first. Then `SherpaVadProvider`; use the same fixtures/clock semantics.
8. Compare with `PRO-014` harness: missed speech, false segment, boundary word loss, CPU/RAM, p50/p95 latency and long-form stability.
9. Only after the VAD POC, evaluate separate Sherpa diarization and generic TTS. They are not a free extension of the VAD change.

### 6.3 Non-negotiable behavior

- If bridge/model fails, recording and current Silero path continue or show a clear safe failure; never silently discard audio.
- “Smart capture” may mark/skip silence for ASR but cannot delete raw data or auto-end a long-form session by default.
- Model bundle size, startup time and license notice are release criteria.
- Do not share speaker embedding across diarization, chat and voice profile subsystems.

## 7. Privacy, security and compliance handoff

### Immediate security backlog facts

1. `SettingsRepository` currently writes provider API keys and custom OpenAI JSON to `settings` / `transcript_settings`. Migrate via `PRO-009` before calendar OAuth/new cloud credentials.
2. Tauri filesystem capabilities are broad. Narrow scopes around recording/session directories and use explicit dialog-selected path capabilities where feasible.
3. `PRIVACY_POLICY.md` says “Last updated: [Current Date]” and contains at-rest encryption wording not directly implemented by the visible SQLite setup. Either implement the claim or revise the policy; do neither by assumption.
4. Existing optional PostHog analytics requires a new check that Pro telemetry never includes title, transcript, audio, speaker label, prompt, calendar metadata or voice profile data.

### GDPR-ready checklist (technical input; legal approval required)

- Data inventory and route disclosure for local, local-LAN/self-hosted and every cloud LLM/calendar provider.
- Purpose/lawful basis/recording notice workflow; specific consent for voice profile, not bundled with normal recording consent.
- Rights flows: access/export, rectify (correction history), delete, retention, revoke consent; test them rather than publish a promise.
- ROPA/DPIA and subprocessor/DPA review if any cloud provider receives content.
- Encryption/key lifecycle based on threat model; no hard-coded key, no secret in logs/backups.
- Audit verifier, clock/tamper handling and authorization controls for future team/self-hosted deployment.
- Correct policy date, language/localisation, supported deployment scope and contact path.

## 8. Voice clone handoff: permitted design only

Treat a voice profile as high-risk/sensitive data even where a particular jurisdiction may define biometric data differently.

**Enrollment must require all of:**

1. authenticated/verified profile owner or an authorized representative workflow;
2. explicit affirmative consent with purpose, geographic/data route, retention, expiry and revoke choice;
3. confirmation that sample is not auto-extracted from a meeting or Dharma recording;
4. consent receipt linked to profile but not containing raw sample;
5. encrypted local artifact and separate key/access control;
6. visible synthetic-audio label in player/export and provenance metadata;
7. delete/revoke command that removes profile, derived cache/model references and future use capability.

**Must be rejected:** third-party/public-figure/teacher cloning without verified authorization, minors, deceptive impersonation, automatic enrollment, reuse of diarization embedding, and unlabeled synthetic export. Watermarking can only be advertised/required where the chosen model has a tested watermark capability; UI label and metadata remain mandatory regardless.

## 9. Test strategy

### Automated layers

| Layer | Required coverage |
|---|---|
| Rust unit | VAD provider clock/flush, model manifest validation, DocumentModel, citation guard, audit hash chain, retention planner. |
| Rust integration | SQLite migration from v0.4.0 fixture, processing run lifecycle, secret-store mock, delete cascade/index invalidation, IPC bridge mocked/fault injected. |
| Frontend unit | Mode/template persistence, citation renderer, export scope, consent disclosure, voice clone refusal states. |
| E2E desktop | Create/import each mode, record/stop/recover, export formats, change ASR run, diarization relabel, chat citation click, calendar disconnect. |
| Audio benchmark | Rights-cleared corpus only; live/batch, VAD boundary, WER/CER/term accuracy/DER/RTF/memory, platform hardware matrix. |
| Security | no-content telemetry test, token/secret scanning, command authorization, path traversal, audit tamper, OAuth revoke, voice revoke/delete. |
| Release | SBOM/NOTICE/model license check, signed installer/update, clean-device install, migration/rollback/recovery smoke. |

### Fixtures

- Store only synthetic or licensed short fixtures in Git.
- Keep full quality corpus in access-controlled storage with manifest IDs/checksums, consent/license status and expiry. CI reports aggregate metrics; no audio/transcript payload uploads.
- Include Vietnamese regional variation, code-switching, Pāli/Sanskrit/Hán–Việt vocabulary, quiet speech, overlap, long pauses and noisy online call fixtures.

### Baseline commands once toolchain exists

```bash
# Frontend dependency/setup — inspect package scripts first and add a test script if needed
cd frontend
pnpm install --frozen-lockfile
pnpm run build

# Existing Rust checks/tests
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The current repository has a few frontend tests using `bun:test` but `package.json` does not define a test script. `PRO-014`/`PRO-029` should establish a single documented test command and CI job rather than relying on ad-hoc local invocations.

## 10. First implementation sprint — recommended pull sequence

Do this after the six M0 cards have owners/approvals:

1. **PR A — no runtime behavior:** ADRs, corpus manifest schema, security inventory, data model diagram; no DB changes.
2. **PR B — schema:** additive session/processing-run migrations + repository/types + old-DB integration fixture.
3. **PR C — secrets:** credential abstraction with platform adapters/mocks, one-time migration, data-redaction tests. Security review before merge.
4. **PR D — session UI/template semantics:** mode picker, metadata editor, bundled online-class/Dharma templates, raw/reviewed text separation.
5. **PR E — analysis bus behind feature flag:** no Sherpa yet; prove existing capture/playback/regression suite.
6. **PR F — VAD interface + Silero adapter:** behavior-equivalent baseline and test fixtures.
7. **PR G — isolated Sherpa bridge POC:** only opt-in developer flag, benchmark report attached to PR.

Keep each PR small enough to rollback. Never combine PR C, E and G: they cross security/audio process boundaries and make failure diagnosis difficult.

## 11. Risk register

| Risk | Likelihood / impact | Mitigation / owner |
|---|---|---|
| Sherpa runtime or model bundle conflicts with current `ort`/Parakeet dependencies | Medium / High | Sidecar first, pinned builds, health/fallback; R/M. |
| VAD misses quiet chanting, reflective silence or low-volume talk | Medium / High | Corpus slices, non-destructive audio, opt-in pause, Silero rollback; M/Q. |
| Diarization after mixed audio produces false labels | High / High | Analysis fan-out before mix, anonymous labels/manual correction; R/M. |
| “High accuracy” claim lacks Vietnamese/domain evidence | High / High | Rights-cleared benchmark and publish limits; P/M. |
| API/OAuth token leakage from SQLite/logs | Medium / Critical | OS key store before new connectors, redaction/security tests; R/S. |
| GDPR marketing overstates local-first architecture | Medium / Critical | DPIA/legal gate, policy claim evidence; S/P. |
| Calendar auto-join violates platform policy or user recording expectation | Medium / High | Reminder + user confirmation only; S/P. |
| Voice clone enables impersonation or harms trust in Dharma content | Medium / Critical | Separate Labs gate, verified consent/revoke/label/red-team; S/P/Q. |
| Public MIT repo cannot protect “Pro-only” code | High / High | Decide packaging in `PRO-001`; private implementation if needed; P/S. |
| Large refactor loses recordings/transcript during stop/reconnect | Medium / Critical | Analysis bus feature flag, fault-injection tests, staged rollout; R/Q. |

## 12. Open decisions requiring product owner response

1. Is Pro open-source commercial support (A) or a separate proprietary application/package (B)?
2. Which OS is GA first: macOS + Windows only, or Linux as a fully supported target? What hardware minimum is acceptable for high-accuracy/diarization/TTS models?
3. Is cloud ASR/LLM permitted, and if yes which providers/regions/data processing terms are allowed?
4. What level of doctrinal review is expected for Dharma templates/glossaries, and who is the reviewer? The product must not imply it replaces a teacher/editor.
5. Is `.ics`/one-click recording sufficient for automation v1, or is actual platform join required despite ToS/legal constraints?
6. Is voice cloning a real commercial requirement or an experimental accessibility feature? Who signs consent, abuse and launch gates?
7. What team/workspace requirements are in v1 versus local single-user only? This changes auth, authorization, audit and GDPR scope significantly.

## 13. Handoff completion checklist

- [x] Repo architecture and current capability gap documented.
- [x] Milestone roadmap, Kanban IDs and dependency order written.
- [x] Sherpa VAD/TTS/diarization path differentiated from voice cloning.
- [x] Meeting, online class and Dharma talk product modes specified.
- [x] Privacy/consent/security/voice safety gates specified.
- [ ] Product owner resolves M0 decisions.
- [ ] Toolchain installed and baseline build/test results captured.
- [ ] First implementation card moved to `In progress` with one DRI.
