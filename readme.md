# SEJASA (Sedekah Jasa) - Backend Engine

Ini adalah repositori core-backend sistem **SEJASA**, sebuah platform berbasis arsitektur *high-performance* menggunakan bahasa **Rust** dan framework **Axum**. Backend ini mengelola penawaran proyek sosial, pencocokan geografis berbasis koordinat, penyimpanan file asinkronus, serta komunikasi *real-time stateful* via WebSockets.

---

## Arsitektur & Arus Data Sistem

Sistem ini mengadopsi pola pemisahan beban kerja (*separation of concerns*) yang agresif untuk menjamin *throughput* tinggi:

1. **Reverse Proxy (NGINX):** Berfungsi sebagai gerbang utama TLS termination, membatasi ukuran muatan (*payload limit*), serta menangani *serving* file statis secara langsung dari disk OS memanfaatkan efisiensi C-level kernel.
2. **Application Server (Axum + Tokio):** Memproses logika bisnis, validasi tipe data yang ketat via *type-state pattern*, enkapsulasi event WebSockets, dan pengelolaan *database connection pool*.
3. **Database Layer (PostgreSQL):** Penyimpanan data persisten yang dioptimasi dengan query relasional kompleks (JSONB aggregation) untuk memangkas overhead pemetaan memori di level aplikasi.

---

## Spesifikasi Fitur Teknis

### 1. Algoritma Pencarian Proyek Terdekat (Geospatial Proximity)

Sistem mengimplementasikan perhitungan matematis rumus **Haversine Formula** langsung di dalam *query relational* PostgreSQL untuk menyaring dan mengurutkan proyek terdekat berdasarkan parameter koordinat (`lat`, `lon`) dan radius maksimal (`distance_meters`) yang dikirimkan oleh klien.

Perhitungan dilakukan dalam satu kali *trip query* tanpa membebani runtime Rust:

```sql
-- Formula Haversine dalam SQL untuk mengukur jarak antara dua titik koordinat bumi
(6371000 * acos(
    cos(radians($1)) * cos(radians(latitude)) * cos(radians(longitude) - radians($2)) + 
    sin(radians($1)) * sin(radians(latitude))
)) AS distance_meters

```

### 2. Full-Text Search & Multi-Param Filtering

Pencarian dan penyaringan proyek pada endpoint `/project` ditangani secara dinamis. Parameter query mendukung:

* **Pencarian Teks (`q`):** Menggunakan klausa `ILIKE %q%` pada judul dan deskripsi proyek.
* **Penyaringan Lanjutan:** Filter instan berdasarkan `status`, `category`, dan pembatasan radius maksimal.
* **Sorting (`most_distance`):** Pengurutan presisi seperti `nearest` (jarak terdekat) memanfaatkan hasil kalkulasi Haversine secara dinamis.

### 3. Penanganan File Asinkronus Berbasis Streaming (Chunking)

Untuk mencegah *Memory Exhaustion (OOM)* akibat pengunggahan file biner berukuran besar, backend menggunakan interpenetrasi **Tokio Async I/O Buffer**.

* **Zero RAM Overloading:** Payload bertipe `multipart/form-data` dibaca sedikit demi sedikit per segmen (*chunk streaming*) menggunakan `while let Some(chunk) = field.chunk().await`.
* **Isolated File System Storage:** File divalidasi ekstensinya (*white-listing* mencegah Stored XSS) lalu di-rename menggunakan `Uuid::new_v4()` sebelum ditulis ke sub-direktori host berdasarkan ID pengguna (`/public/uploads/{user_id}/`).
* **Shared Storage Volume:** Sinkronisasi instan antara Axum (Akses Tulis) dan NGINX (Akses Baca) dikelola menggunakan Docker Bind Mounts.

### 4. Real-time Communication Stateful (WebSockets)

Fitur obrolan interaktif mengandalkan protokol WebSocket (`axum::extract::ws`) untuk meminimalisir overhead HTTP:

* **Auto-Load History:** Saat koneksi terbuka, sistem melakukan *eager-loading* riwayat chat lalu mengirimkannya ke klien sebelum melakukan proses *stream splitting* (`sender` & `receiver`).
* **Atomic Read Updates:** Setelah riwayat dikirim, backend menembakkan perintah *bulk update* asinkronus ke database untuk mengubah seluruh pesan milik lawan bicara menjadi `is_read = true` dalam satu operasi database tunggal (`WHERE chat_id = $1 AND sender_id != $2 AND is_read = false`).

---

## Standarisasi Response & Endpoint API

Semua endpoint API (baik sukses maupun gagal) wajib mematuhi skema JSON standar industri untuk mempermudah parsing tipe data statis di sisi klien:
### **Dokumentasi API (Postman)**: [Lihat di Postman](https://www.postman.com/teknohole/workspace/sejasa)

### Contoh Struktur Data Response (Sukses dengan Paginasi)

```json
{
    "success": true,
    "status": 200,
    "message": "List projects!",
    "path": "/project",
    "timestamp": "2026-05-21T14:59:44Z",
    "meta": {
        "current_page": 1,
        "limit_page": 5,
        "total_items": 2,
        "total_pages": 1
    },
    "data": [
        {
            "id": "019e3656-6624-742d-b367-ea64e10c7f50",
            "name": "Pengembangan Sistem Greenhouse Duck Egg",
            "status": "hiring",
            "rating": 4.8,
            "descriptions": "Membangun otomatisasi inkubasi telur bebek...",
            "slug": "pengembangan-sistem-greenhouse-duck-egg",
            "address": "Yogyakarta, Indonesia",
            "latitude": -7.784130,
            "longitude": 110.402604,
            "distance_meters": 12.50,
            "max_participant": 5,
            "requirements": {
                "skills": ["Rust", "Embedded C", "IoT"]
            },
            "category": {
                "id": "019e3656-abcd-efgh-b367-ea64e10c7f50",
                "name": "Testing"
            },
            "owner": {
                "id": "019e3656-9999-8888-b367-ea64e10c7f50",
                "name": "Bakutani Tech",
                "image": "/uploads/9999/avatar.png"
            },
            "updated_at": "2026-05-21T10:00:00Z",
            "created_at": "2026-05-21T08:00:00Z"
        }
    ]
}

```

---

## Kontainerisasi & Orkestrasi (Docker)

Backend dikemas menggunakan multi-container Docker Compose. NGINX diletakkan di depan sebagai pelindung internal network container Axum.

### Konfigurasi `docker-compose.yml`

```yaml
services:
  axum-sejasa:
    build: .
    container_name: axum-sejasa
    restart: unless-stopped
    env_file: .env
    ports:
      - "8010:8010"
    extra_hosts:
      - "host.docker.internal:host-gateway"
    volumes:
      - ./public/uploads:/app/public/uploads

  nginx-sejasa:
    image: nginx:alpine
    container_name: nginx-sejasa
    restart: unless-stopped
    ports:
      - "80:80"
    depends_on:
      - axum-sejasa
    volumes:
      - ./nginx.conf:/etc/nginx/conf.d/default.conf:ro
      - ./public/uploads:/usr/share/nginx/html/uploads:ro

```

---

## Langkah Memulai & Instalasi

### Prasyarat Sistem

* Rust Compiler / Toolchain (MSRV 1.75+)
* PostgreSQL Database v14+
* Docker & Docker Compose (Untuk deployment instan)

### Langkah Instalasi Lokal (Tanpa Docker)

1. **Clone Repositori Backend:**
```bash
git clone https://github.com/Arbath/sejasa_be.git
cd sejasa_be

```


2. **Konfigurasi Environment Variable:**
Buat file `.env` di direktori utama:
```env
APP_PORT=8010
# Options: DEBUG, INFO, WARN, ERROR, TRACE
LOG_LEVEL=DEBUG
CORS_ORIGINS=http://127.0.0.1:8011,http://localhost:5173
JWT_SECRET=secret
# Default: 15 minutes
ACCESS_TTL_IN_MINUTES=15
# Default: 7 days
REFRESH_TTL_IN_DAYS=7
DATABASE_URL=postgres://username:password@localhost:5432/database
# for postgresql in host machine
# DATABASE_URL=postgres://username:password@host.docker.internal:5432/database
DB_CONNECTIONS=10

# Set true for auto migrations
# MIGRATE=true

```


3. **Eksekusi Migrasi Database:**
Pastikan Anda telah menginstal `sqlx-cli`, lalu jalankan migrasi schema jika tidak menggunakan auto migration:
```bash
sqlx database setup

```


4. **Kompilasi & Jalankan Server:**
```bash
cargo run --release

```



### Langkah Instalasi Berbasis Docker (Production-Ready)

1. Siapkan struktur folder lokal untuk volume sharing agar terhindar dari *permission issue*:
```bash
mkdir -p public/uploads

```


2. Bangun citra (*build image*) dan nyalakan seluruh infrastruktur kontainer di latar belakang:
```bash
docker compose up -d --build

```


3. Pastikan semua log berjalan normal:
```bash
docker compose logs -f

```

---