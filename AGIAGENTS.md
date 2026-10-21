# QUY TẮC HOẠT ĐỘNG & ĐỒNG BỘ DỰ ÁN (AGI AGENTS GUIDELINES)

Tài liệu này lưu giữ toàn bộ các quy tắc và nguyên tắc bắt buộc đối với tất cả AI Agents, Developer, và Contributor làm việc trên dự án **Universal Autonomous AI Workspace**.

---

## I. NGUYÊN TẮC ĐỒNG BỘ GITHUB BẮT BUỘC (MỨC ĐỘ CAO NHẤT)

1. **Cam kết và Đẩy mã nguồn theo từng thay đổi nhỏ nhất (Micro-Commits & Continuous Push)**:
   - Mỗi lần hoàn thành xong **bất kỳ 1 tính năng nào dù là nhỏ nhất** (thêm/sửa 1 nút bấm, cập nhật giao diện, sửa 1 lỗi logic, cập nhật tài liệu, tối ưu hóa hàm, thêm cấu hình...), AI Agent **bắt buộc phải thực hiện ngay `git commit` và `git push` lên GitHub**.
   - **Tuyệt đối không dồn việc**: Không đợi làm xong nhiều tính năng hoặc toàn bộ yêu cầu lớn mới commit và push 1 lần.
   - **Tần suất trong một prompt**: Trong cùng một câu lệnh / prompt của người dùng, AI Agent **hoàn toàn có thể và được khuyến khích commit & push lên GitHub nhiều lần** tương ứng với từng bước tiến độ đạt được.

2. **Quy trình thực thi**:
   - Bước 1: Kiểm tra tính toàn vẹn (chạy build/test nhanh nếu liên quan đến logic).
   - Bước 2: `git status` kiểm tra file thay đổi.
   - Bước 3: `git add <các file đã sửa/thêm>`
   - Bước 4: `git commit -m "<thông điệp mô tả ngắn gọn, súc tích chuẩn Conventional Commits>"`
   - Bước 5: `git push origin master:main` (hoặc branch theo dõi tương ứng).
   - Bước 6: Xác nhận trạng thái `Your branch is up to date with 'origin/main'`.

---

## II. KIẾN TRÚC & TIÊU CHUẨN KỸ THUẬT CỐT LÕI

1. **Kiến trúc Decoupled & Zero Regression**:
   - Không được phép làm hỏng hoặc xóa bỏ các tính năng đã hoạt động từ các Phase 1 đến Phase 6.
   - Rust Core (`agent-platform`, `agent-security`, `agent-storage`, `agent-tools`, `agent-core`, `agent-runtime`) phải luôn duy trì 100% test pass (`cargo test`).
   - Đảm bảo độc lập: Nền tảng có thể vận hành ở chế độ Windows Service ngầm (SCM) không phụ thuộc cửa sổ Desktop UI.

2. **Tiêu chuẩn Windows Native & Desktop UI**:
   - Ẩn hoàn toàn console: File thực thi Desktop (`desktop.exe`) phải sử dụng `#![windows_subsystem = "windows"]`.
   - Mọi tiến trình phụ trợ spawn ngầm (Powershell, CMD, Python, uv, taskkill) bắt buộc phải kèm cờ Win32 `CREATE_NO_WINDOW` (`0x08000000`) để không flash cửa sổ đen ra màn hình người dùng.
   - Luôn nhúng sẵn bundle frontend tĩnh (`dist/`) vào file nhị phân qua `frontendDist: "../dist"` để ứng dụng mở tức thì độc lập (standalone), loại bỏ hoàn toàn lỗi `ERR_CONNECTION_REFUSED`.
   - Hỗ trợ đầy đủ ngôn ngữ **Tiếng Việt 🇻🇳** và **English 🇺🇸** với khả năng chuyển đổi tức thì trong Cài đặt (`Settings.tsx`) và lưu trữ trạng thái tại `localStorage`.
