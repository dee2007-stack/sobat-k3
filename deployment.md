# Dokumentasi Deployment Production — Sobat K3

- **URL Production**: `https://www.bpombandung.services/sobat-k3/`
- **Direktori Proyek**: `/var/www/html/sobatk3`
- **Symlink Web Root**: `/var/www/html/sobat-k3` -> `/var/www/html/sobatk3/frontend/dist`
- **Tanggal**: 18 November 2026

## 1. Arsitektur Layanan
- **Reverse Proxy Lapis 1 (Nginx)**: Terminasi TLS/HTTPS di port `443` (`bpombandung.services` / `www.bpombandung.services`), meneruskan trafik ke Apache di `127.0.0.1:8081`.
- **Web Server Lapis 2 (Apache + `.htaccess`)**:
  - Menyajikan static bundle Leptos WASM dari `/var/www/html/sobat-k3/`.
  - Mem-proxy `/sobat-k3/api/*` ke Rocket backend (`http://127.0.0.1:8010/api/$1`).
  - Menyediakan SPA fallback ke `index.html`.
- **Backend (Rust + Rocket 0.5.1 + SQLx 0.8)**: Berjalan di `127.0.0.1:8010` dikelola oleh PM2 (`sobatk3-api`).
- **Database (PostgreSQL)**: Database `sobat_k3` (owner `dee`), koneksi melalui Unix domain socket.

## 2. Database PostgreSQL
Database dibuat dengan perintah:
```bash
createdb sobat_k3
```
Verifikasi koneksi Unix socket (peer auth sebagai user `dee`):
```bash
psql "postgres:///sobat_k3?host=/var/run/postgresql" -c "SELECT 1;"
```

## 3. Konfigurasi `.env` Production
Lokasi: `/var/www/html/sobatk3/.env`
```env
DATABASE_URL=postgres:///sobat_k3?host=/var/run/postgresql
JWT_SECRET=<64-char-hex-secret>
ROCKET_PORT=8010
ROCKET_ADDRESS=127.0.0.1
ROCKET_PROFILE=release
UPLOAD_DIR=/var/www/html/sobatk3/uploads
```
> **Catatan**: Port `8000` sudah digunakan oleh `php8.2`, sehingga backend menggunakan `127.0.0.1:8010`. Format `postgres:///sobat_k3?host=/var/run/postgresql` digunakan agar parser URL `sqlx` tidak mengalami error `Configuration(EmptyHost)`.

## 4. Konfigurasi `.htaccess` & Keamanan
1. **Proteksi Direktori Source** (`/var/www/html/sobatk3/.htaccess`):
   Menutup akses langsung ke source code, `.env`, `backend/`, dan `uploads/` dengan `Require all denied`.
2. **Routing & Security Headers Web Root** (`/var/www/html/sobatk3/frontend/.htaccess` -> disalin otomatis ke `frontend/dist/.htaccess` oleh Trunk):
   - Menolak akses file tersembunyi (`<FilesMatch "^\."> Require all denied`).
   - Menambahkan header `X-Frame-Options`, `X-Content-Type-Options`, `Referrer-Policy`, dan `X-XSS-Protection`.
   - Reverse proxy `^api/(.*)$` ke `http://127.0.0.1:8010/api/$1 [P,L,QSA]`.
   - SPA fallback ke `index.html`.
   - Didaftarkan di `frontend/index.html` melalui `<link data-trunk rel="copy-file" href=".htaccess" />` agar tidak terhapus saat `trunk build --release`.

## 5. Penyesuaian Source Code
- **`backend/src/main.rs`**: Membaca `ROCKET_ADDRESS` (default `127.0.0.1`), membatasi `Access-Control-Allow-Origin` ke `https://www.bpombandung.services` / `https://bpombandung.services`, serta menambahkan header `X-Content-Type-Options: nosniff` dan `X-Frame-Options: SAMEORIGIN`.
- **`frontend/Trunk.toml`**: Menambahkan `public_url = "/sobat-k3/"`.
- **`frontend/src/api.rs`**: Menambahkan prefix `API_BASE` (`/sobat-k3`) untuk seluruh endpoint `/api/...`.
- **`frontend/src/app.rs` & `frontend/src/components/layout.rs`**: Menggunakan path eksplisit `/sobat-k3/...` pada seluruh `<Route>` dan `<A href="...">` agar navigasi menu tetap berada di subpath `/sobat-k3/`.
- **`frontend/src/components/layout.rs`**: Menambahkan `min-w-0 w-full overflow-x-hidden` pada `<main class="flex-1 ...">` agar konten utama tidak melebar melebihi viewport desktop/laptop akibat tabel 14 kolom.

## 6. Build & Operasional (PM2)
```bash
# Build Frontend WASM
cd /var/www/html/sobatk3/frontend && trunk build --release

# Build Backend Release
cd /var/www/html/sobatk3 && cargo build --release -p backend

# Jalankan / Restart di PM2
pm2 start /var/www/html/sobatk3/target/release/backend --name sobatk3-api --cwd /var/www/html/sobatk3
pm2 save
```

## 7. Verifikasi
- `GET https://www.bpombandung.services/sobat-k3/` → `HTTP 200`
- `POST https://www.bpombandung.services/sobat-k3/api/auth/login` → `HTTP 200` (JWT valid)
- Tampilan desktop/laptop responsif tanpa horizontal overflow.
