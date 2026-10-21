# Kiến Trúc Giao Diện Chat-First & Trải Nghiệm Desktop Tối Giản (Chat-First Minimalist Desktop Architecture)

Tài liệu này ghi lại chi tiết triết lý thiết kế, cấu trúc thành phần và quy chuẩn trải nghiệm người dùng của ứng dụng **Universal Autonomous AI Workspace** sau đợt tái thiết kế 100% giao diện sang mô hình **Chat-First**.

---

## I. TRIẾT LÝ THIẾT KẾ CỐT LÕI

### 1. Chat-First (Trung tâm Hội thoại Duy nhất)
Toàn bộ ứng dụng hội tụ về một không gian làm việc duy nhất: **Chat với AI**. Người dùng không cần phải chuyển đổi giữa hàng loạt trang Dashboard hay quản trị phức tạp. Mọi yêu cầu từ viết code, kiểm thử, phân tích tài liệu, điều khiển trình duyệt, thao tác ứng dụng Windows cho đến kích hoạt workflow nhiều bước đều được thực hiện trực tiếp từ dòng chat.

### 2. Invisible Complexity (Độ Phức Tạp Ẩn)
Hệ thống ngầm vẫn duy trì toàn bộ sức mạnh của các Agent (Coding Agent, Computer Agent, Browser Agent, Research Agent, Workflow DAGs, Windows MCP, SQLite, 24/7 Service), nhưng giao diện người dùng hoàn toàn tĩnh lặng và không làm phiền người dùng bằng các thông số KPI, biểu đồ đo đạc, hay các cấu hình thủ công không cần thiết. AI tự động lựa chọn công cụ và thông báo tiến độ theo ngữ cảnh.

### 3. Chủ nghĩa Tối giản (Minimalism)
- Loại bỏ hoàn toàn các hiệu ứng glow AI, gradient sặc sỡ, card KPI dày đặc.
- Sử dụng các đường viền siêu mỏng (1px border), khoảng trắng cân đối, typography rõ ràng.
- Hỗ trợ đầy đủ **Chế độ Sáng (Light Mode)**, **Chế độ Tối (Dark Mode)** và **Tự động theo hệ điều hành (System Mode)**.

---

## II. BẢNG SO SÁNH THÀNH PHẦN (TRƯỚC & SAU TÁI CẤU TRÚC)

| Màn hình / Thành phần Cũ | Nhược điểm Cũ | Giải pháp Mới (Chat-First) |
| :--- | :--- | :--- |
| `Dashboard.tsx` | Quá nhiều KPI card, token stats, agent stats rối mắt | Loại bỏ hoàn toàn. Ứng dụng mở thẳng vào giao diện chat. |
| `Sidebar.tsx` (13 tab) | Phải bấm chuyển qua lại giữa 13 màn hình rời rạc | Thay bằng `ConversationSidebar.tsx` chỉ quản lý lịch sử chat và tìm kiếm. |
| `EmergencyStopBanner.tsx` | Dải đỏ chói mắt chiếm diện tích trên đỉnh mỗi trang | Tích hợp thành nút pill tinh tế trên `WindowHeader.tsx` chỉ hiện khi kích hoạt. |
| `CodingWorkspace.tsx` | Giao diện IDE cồng kềnh với Ask/Plan/Build | Code được AI tạo, hiển thị qua `CodeBlock.tsx` với nút sao chép và cú pháp đẹp mắt. |
| `TerminalView.tsx` | Cửa sổ dòng lệnh tách biệt | Lệnh chạy được hiển thị dạng thẻ `ContextualActivity.tsx` có thể mở rộng xem stdout/stderr. |
| `ComputerControl.tsx` | Bảng điều khiển cửa sổ desktop riêng | Các hành động chiếm quyền điều khiển desktop hiển thị trực tiếp trong dòng tin nhắn. |
| `BrowserWorkspace.tsx` | Khung xem trình duyệt riêng | Các bước duyệt web hiển thị trực tiếp trong dòng tin nhắn dạng activity widget. |
| `ApprovalDialog.tsx` | Modal pop-up chặn toàn màn hình gây giật mình | Thẻ phê duyệt nội dòng `InlineApprovalCard.tsx` hiển thị tự nhiên trong luồng chat. |
| `Settings.tsx` (10 tab) | Bảng cài đặt khổng lồ, quá nhiều thông số kỹ thuật | Hộp thoại tối giản `SettingsDialog.tsx` với 3 tab: Giao diện, Nhà cung cấp AI, Hệ thống. |

---

## III. CHI TIẾT CÁC THÀNH PHẦN GIAO DIỆN MỚI

### 1. `WindowHeader.tsx` (Thanh tiêu đề Cửa sổ Tối giản)
- Chiều cao cố định 44px, viền mỏng phía dưới.
- Bên trái: Nút đóng/mở thanh bên (`PanelLeft`), Tiêu đề ứng dụng ("SIA") kèm tên cuộc trò chuyện hiện tại.
- Bên phải:
  - Nút "Cuộc trò chuyện mới" (`Plus`).
  - Nút cảnh báo Dừng khẩn cấp (chỉ hiển thị nhấp nháy khi đang kích hoạt).
  - Nút mở Cài đặt (`Settings2`).

### 2. `ConversationSidebar.tsx` (Thanh bên Lịch sử Trò chuyện)
- Có thể thu gọn mở rộng mượt mà (`w-64` sang `w-0` với hiệu ứng chuyển đổi không giật layout).
- Nút "Cuộc trò chuyện mới" đặt nổi bật ở đầu.
- Ô tìm kiếm thời gian thực theo tiêu đề.
- Danh sách hội thoại phân nhóm theo mốc thời gian:
  - **Hôm nay (Today)**
  - **Hôm qua (Yesterday)**
  - **7 ngày trước (Previous 7 Days)**
  - **Cũ hơn (Older)**
- Thao tác nhanh khi di chuột: Đổi tên (`Edit2`), Xóa kèm xác nhận an toàn (`Trash2`).
- Chân trang hiển thị thư mục Workspace hiện tại và lối tắt cài đặt.

### 3. `ChatView.tsx` (Không gian Hội thoại & Dòng Tin nhắn)
- **Trạng thái Trống (Empty State)**: Lời chào tự nhiên *"Tôi có thể giúp gì cho bạn?"* kèm 4 thẻ gợi ý hành động nhanh (Kiểm tra & build code, Lệnh terminal, Nghiên cứu tài liệu, Tự động hóa).
- **Trạng thái Hoạt động**:
  - Tự động cuộn xuống khi có tin nhắn mới hoặc streaming.
  - Nhận diện khi người dùng chủ động cuộn ngược lên để tắt tự động cuộn, đồng thời hiện nút bấm nổi *"Cuộn xuống dưới cùng ↓"*.
  - Hiển thị tiến trình suy nghĩ của AI (`ModelThinking`) và các thẻ hoạt động công cụ đang chạy.

### 4. `ChatComposer.tsx` (Khung Soạn Thảo Tin Nhắn Thông Minh)
- Tự động co giãn chiều cao theo nội dung nhập (tối thiểu 42px, tối đa 200px).
- Hỗ trợ gửi tin nhắn bằng phím `Enter`, xuống dòng bằng `Shift + Enter`.
- **Bảo vệ Bộ gõ Tiếng Việt (IME Composition Guard)**: Lắng nghe sự kiện `compositionstart` và `compositionend` để ngăn việc gửi nhầm tin nhắn khi người dùng đang gõ phím bỏ dấu (Telex, VNI).
- Nút đính kèm tệp và hiển thị danh sách tệp đính kèm dạng badge.
- Nút Dừng (`Square`) khi AI đang tạo phản hồi, và nút Gửi (`ArrowUp`) khi sẵn sàng.
- Badge hiển thị mô hình AI đang kết nối.

### 5. `ContextualActivity.tsx` (Thẻ Hoạt động Ngữ cảnh Nội dòng)
- Thay thế hoàn toàn cho các màn hình phụ và terminal rời.
- Hiển thị biểu tượng tương ứng với từng công cụ (Terminal, File, Browser, Desktop).
- Trạng thái rõ ràng: Đang chạy (`Loader2`), Thành công (`CheckCircle2`), Thất bại (`XCircle`).
- Có thể bấm để mở rộng xem các tham số đầu vào (JSON) và kết quả đầu ra (stdout, stderr, exit code).

### 6. `InlineApprovalCard.tsx` (Thẻ Phê duyệt Quyền Nội dòng)
- Hiển thị trực tiếp trong luồng chat khi AI yêu cầu các quyền nhạy cảm (ghi tệp ngoài workspace, chạy lệnh shell hủy diệt, chiếm quyền chuột/bàn phím).
- Cho phép người dùng bấm **Chấp thuận (Approve)** hoặc **Từ chối (Reject)** ngay tại chỗ mà không bị ngắt quãng trải nghiệm hội thoại.

### 7. `SettingsDialog.tsx` (Hộp thoại Cài đặt Tối giản)
- Hiển thị dạng modal popover tinh tế với nền mờ backdrop.
- **Tab 1: Giao diện (Appearance)**: Chọn chế độ Sáng / Tối / Tự động, Chuyển đổi ngôn ngữ Tiếng Việt 🇻🇳 / English 🇺🇸.
- **Tab 2: Mô hình AI (AI Provider)**: Cấu hình Endpoint, Tên Model, API Key (ẩn/hiện), Nút kiểm tra kết nối thời gian thực.
- **Tab 3: Hệ thống & Quyền (System & Security)**: Thư mục Workspace, Chế độ phân quyền (Strict / WorkspaceAuto / FullAuto), Trạng thái dịch vụ nền 24/7.

---

## IV. BẢO TOÀN TÍNH NĂNG BACKEND & CHUẨN ĐÓNG GÓI

- Mã nguồn frontend đã được dọn sạch 100% các file dư thừa, giảm kích thước bundle CSS xuống còn **21.25 kB** và JS còn **204.81 kB**.
- Toàn bộ backend Rust Core (Phase 1 đến Phase 6) gồm 34/34 bài kiểm thử chấp nhận đều vượt qua (`passed`).
- Bundle tĩnh đã được nhúng hoàn toàn vào file thực thi nhị phân `desktop.exe`, đảm bảo khởi động tức thì, hoạt động độc lập (offline/standalone) và không có bất kỳ cửa sổ CMD nhấp nháy nào.
