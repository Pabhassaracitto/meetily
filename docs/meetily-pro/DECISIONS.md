# Meetily Pro — các quyết định đã chốt

> **Cập nhật:** 2026-08-08 (Asia/Colombo)
> **Nguồn:** lựa chọn của product owner trong phiên lập kế hoạch.

| ID | Quyết định | Trạng thái | Hệ quả triển khai |
|---|---|---|---|
| `D-001` | Toàn bộ Meetily Pro sẽ **open source** trong repo hiện tại. | Chốt | Không xây ranh giới proprietary/private plugin. Cần làm rõ mô hình doanh thu bằng support, bundle, hosting hoặc dịch vụ; entitlement không là blocker cho Pro Core. |
| `D-002` | Bản phát hành đầu tiên là **Pro Core cho lớp học & pháp thoại**. | Chốt | Ưu tiên accuracy, session modes, glossary, templates, Markdown/DOCX/PDF và privacy foundation. Chat/diarization là beta sau Core. |
| `D-003` | **Local mặc định, cloud tùy chọn**. | Chốt | Existing provider routes vẫn được hỗ trợ, nhưng mọi cloud route phải có disclosure/consent; secret migration vẫn là bắt buộc trước calendar/cloud expansion. |
| `D-004` | Làm **generic TTS trước**, clone giọng để sau. | Chốt | Voice cloning không thuộc Pro Core; generic local TTS chỉ bắt đầu sau foundations và model/license evaluation. |

## Những gì vẫn chưa được chốt

- Platform GA và minimum hardware.
- Corpus có quyền sử dụng, reviewer cho thuật ngữ/pháp thoại.
- Calendar provider/scope và mức automation sau one-click record.
- Encryption-at-rest implementation cụ thể, legal/DPIA sign-off và retention defaults.

Các mục chưa chốt là gate của cards tương ứng, không phải lý do để trì hoãn vertical slice Session Type + domain templates.

## Kickoff slice đã bắt đầu

Bản thay đổi đầu tiên sau các quyết định này triển khai nền session mode theo hướng additive:

- migration `20260808000000_add_session_type.sql` với default `meeting` cho dữ liệu cũ;
- lựa chọn **Meeting / Online Class / Dharma Talk** khi ghi âm hoặc import;
- lưu session type khi persist/recover transcript; API metadata trả type để UI hiển thị;
- template built-in `online_class` và `dharma_talk`, tự chọn theo session type khi mở summary;
- lựa chọn template được persist theo session thay vì chỉ nằm trong React state, nên reload/tạo summary lại vẫn giữ workflow đã chọn.

Đây mới là vertical slice M1, chưa phải claim hoàn tất processing provenance, glossary, export, audit hoặc compliance controls.

## Quality foundation đang triển khai

Đã thêm [`benchmarks/`](../../benchmarks/README.md) cùng `scripts/benchmark_transcription.py` để so sánh WER/CER, term accuracy, latency và real-time factor mà không đưa raw transcript/audio vào report Git. Harness đã có unit test; corpus tiếng Việt/lớp học/pháp thoại có quyền sử dụng và ngưỡng phát hành vẫn là gate bắt buộc của `PRO-003`.

## Processing provenance đang triển khai

Mỗi transcript được lưu từ live capture, import, retranscription hoặc recovery giờ tạo một `processing_runs` record trong cùng transaction với transcript. Record lưu source, provider/model, language hint, VAD config, thời gian xử lý và aggregate metrics; repository từ chối các key nội dung như `transcript`, `text`, `audio`, `prompt`, API key hoặc embedding trong metadata. API `api_get_processing_runs` đã sẵn sàng cho UI history/compare ở milestone tiếp theo.
