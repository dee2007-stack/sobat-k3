# Sobat K3 — Aplikasi Manajemen Rencana & Program Kerja K3L / P2K3 BBPOM di Bandung

Aplikasi full-stack **100% Rust** untuk digitalisasi **Rencana & Program Kerja K3L / P2K3 BBPOM di Bandung Tahun 2027** (terintegrasi SMPOMT, SMK3 PP 50/2012, ISO 45001:2018 & ISO 14001:2026).

## Arsitektur & Teknologi
- **Shared (`shared/`)**: Model DTO, enum Role RBAC, dan rumus K3 (`Safe Manhours`, `FR`, `SR`, `TRIR`, `Capaian %`).
- **Backend (`backend/`)**: Rust + **Rocket 0.5.1** + **SQLx 0.8** + **PostgreSQL** + Argon2 + JWT + `rust_xlsxwriter`.
- **Frontend (`frontend/`)**: Rust + **Leptos 0.7 (WASM CSR)** + **Tailwind CSS** + **Shadcn UI** (Mobile-First & Responsif).

## Fitur Utama
1. **Autentikasi & RBAC (4 Role)**:
   - `Admin`: Kelola seluruh data K3L + Menu Admin (CRUD Pengguna, Role, Reset Password, Audit Log).
   - `Ketua Tim K3`: Full CRUD program/indikator + Tanda Tangan Pengesahan "Disiapkan Oleh".
   - `Sekretaris Tim K3`: Full CRUD program/indikator + Verifikasi "Diverifikasi Oleh".
   - `Anggota Tim K3`: Lihat matriks & dashboard, input realisasi bulanan serta pelaporan insiden/nearmiss.
2. **Tabel 1: Jam Kerja Efektif & Safe Manhours (2027)** — 242 hari kerja efektif (SKB 3 Menteri) dengan kalkulasi otomatis jam kerja harian & kumulatif.
3. **Tabel 2: Matriks TSP (Pilar A–D)** — 25 program kerja awal (A.1.1 s.d. D.4.2), mode Kartu Mobile-First & mode Tabel Matriks Lengkap, input realisasi bulanan, analisis akar masalah, dan tindakan perbaikan/pencegahan.
4. **Bagian E: Lagging Indicators & Statistik K3L** — 15 parameter standar dengan kalkulasi otomatis `Safe Manhours`, `FR`, `SR`, dan `TRIR`.
5. **Pengesahan & Ekspor**: Blok pengesahan berjenjang, ekspor **Excel (`.xlsx`)** multi-sheet, dan cetak **PDF**.

## Cara Menjalankan
1. Pastikan PostgreSQL aktif dan buat database `sobat_k3`:
   ```bash
   createdb sobat_k3
   ```
2. Salin `.env.example` ke `.env` dan sesuaikan `DATABASE_URL` serta `JWT_SECRET`.
3. Jalankan Backend (otomatis menjalankan migrasi & seeder data 2027):
   ```bash
   cargo run -p backend
   ```
4. Jalankan Frontend WASM (di terminal lain):
   ```bash
   cd frontend && trunk serve
   ```
5. Buka browser di `http://127.0.0.1:8080`.

## Akun Bawaan (Seeder)
Password default untuk semua akun demo: `SobatK3!2027`
- `admin` (Admin)
- `ketua_k3` (Ketua Tim K3)
- `sekretaris_k3` (Sekretaris Tim K3)
- `anggota_k3` (Anggota Tim K3)
