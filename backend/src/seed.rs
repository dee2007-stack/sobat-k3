use crate::auth::hash_password;
use sqlx::PgPool;
use uuid::Uuid;

struct SeedProgram {
    pillar: &'static str,
    pillar_title: &'static str,
    subcategory: &'static str,
    item_number: &'static str,
    objective: &'static str,
    measurable_target: &'static str,
    code: &'static str,
    description: &'static str,
    frequency: &'static str,
    reference_law: &'static str,
    pic: &'static str,
    output_evidence: &'static str,
    annual_target: f64,
    is_target_auto_sum: bool,
    months: [(&'static str, f64); 12],
}

pub async fn run_seed(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 1. Seed Users
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if user_count.0 == 0 {
        let default_pass = hash_password("SobatK3!2027").expect("Failed to hash default password");
        let default_users = [
            (
                "admin",
                "admin@bbpom-bandung.go.id",
                "Administrator Sistem Sobat K3",
                "199001012020011001",
                "Admin Sistem K3L",
                "admin",
            ),
            (
                "ketua_k3",
                "ketua.p2k3@bbpom-bandung.go.id",
                "Ketua Tim P2K3 BBPOM Bandung",
                "198005152008011002",
                "Ketua Tim P2K3 BBPOM di Bandung",
                "ketua_tim_k3",
            ),
            (
                "sekretaris_k3",
                "sekretaris.p2k3@bbpom-bandung.go.id",
                "Sekretaris Tim P2K3 (Ahli K3)",
                "198708202014022003",
                "Sekretaris Tim P2K3 (Ahli K3)",
                "sekretaris_tim_k3",
            ),
            (
                "anggota_k3",
                "anggota.p2k3@bbpom-bandung.go.id",
                "Anggota Tim P2K3 / Pengelola Lab",
                "199303102019031004",
                "Anggota Tim P2K3 / Koordinator Lab",
                "anggota_tim_k3",
            ),
        ];

        for (username, email, full_name, nip, position_title, role) in default_users {
            sqlx::query(
                "INSERT INTO users (username, email, full_name, nip, position_title, role, password_hash)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(username)
            .bind(email)
            .bind(full_name)
            .bind(nip)
            .bind(position_title)
            .bind(role)
            .bind(&default_pass)
            .execute(pool)
            .await?;
        }
    }

    // 2. Seed Document Period 2027
    let period_exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM document_periods WHERE year = 2027")
            .fetch_optional(pool)
            .await?;

    if period_exists.is_some() {
        return Ok(());
    }

    let period_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO document_periods (
            id, year, title, subtitle, doc_number, revision, sk_basis, legal_basis,
            approval_date_text, chair_name, chair_nip, secretary_name, secretary_nip, head_name, head_nip, status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 'draft')"
    )
    .bind(period_id)
    .bind(2027)
    .bind("RENCANA & PROGRAM KERJA K3L / P2K3 BBPOM DI BANDUNG TAHUN 2027")
    .bind("Balai Besar Pengawas Obat dan Makanan di Bandung • Jl. Pasteur No. 25, Kota Bandung • Terintegrasi SMPOMT, SMK3 PP 50/2012, ISO 45001:2018 & ISO 14001:2026")
    .bind("TSP-P2K3-BBPOM-BDG-2027")
    .bind("01 (Final)")
    .bind("Keputusan Kepala BBPOM di Bandung Nomor HK.02.02.8A.01.26.97 Tahun 2026")
    .bind("UU 1/1970, UU 27/2026, PP 50/2012, Permenaker 13/2025, Permenaker 5/2018, PP 22/2021, Kepka BPOM HK.02.02.1.2.06.21.251/2021, Kepka BPOM 289 & 290/2026")
    .bind("Bandung, .................... 2026")
    .bind("Ketua Tim P2K3 BBPOM di Bandung")
    .bind("")
    .bind("Sekretaris Tim P2K3 (Ahli K3)")
    .bind("")
    .bind("Kepala Balai Besar POM di Bandung")
    .bind("")
    .execute(pool)
    .await?;

    // 3. Seed Work Hours 2027 (SKB 3 Menteri = 242 hari kerja efektif)
    let effective_days_2027 = [19.0, 19.0, 17.0, 22.0, 17.0, 21.0, 22.0, 21.0, 22.0, 21.0, 22.0, 19.0];
    for (idx, days) in effective_days_2027.iter().enumerate() {
        sqlx::query(
            "INSERT INTO work_hours_monthly (period_id, month, effective_days, standard_hours_per_day, avg_employees_present)
             VALUES ($1, $2, $3, 8.0, 0.0)",
        )
        .bind(period_id)
        .bind((idx + 1) as i32)
        .bind(days)
        .execute(pool)
        .await?;
    }

    // 4. Seed Work Programs (Pilar A, B, C, D)
    const PILAR_A: &str = "A. KESELAMATAN KERJA (SAFETY) — MEMENUHI 5 UNSUR WAJIB PROGRAM P2K3 (PASAL 10 PERMENAKER 13/2025)";
    const PILAR_B: &str = "B. KESEHATAN KERJA, HIGIENE INDUSTRI & SANITAIR (HEALTH)";
    const PILAR_C: &str = "C. LINGKUNGAN HIDUP & PENGELOLAAN LIMBAH B3 (ENVIRONMENT)";
    const PILAR_D: &str = "D. MUTU LAYANAN K3, SARANA PRASARANA & LINGKUNGAN KERJA INKLUSIF";

    let programs = vec![
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "", item_number: "1",
            objective: "Kepatuhan terhadap Peraturan Perundang-undangan K3",
            measurable_target: "Kepatuhan regulasi K3: 100%\nPelaporan 6 bulanan tepat waktu: 100%",
            code: "A.1.1",
            description: "Evaluasi pemenuhan regulasi K3, pembaruan SK P2K3 (maks. 2 hari kerja bila ada perubahan Ketua/Sekretaris), dan penyampaian Laporan Berkala 6 Bulanan P2K3 (format 5 bagian) ke Disnakertrans Prov. Jabar & Kemnaker",
            frequency: "1x per 6 bulan",
            reference_law: "Permenaker 13/2025 Ps. 8 & 11; Kepka BPOM HK.02.02.1.2.06.21.251/2021 Ps. 33",
            pic: "Ketua & Sekretaris P2K3",
            output_evidence: "Laporan Semester I & II + Tanda Terima",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "I. PENGELOLAAN FAKTOR MANUSIA & KOMPETENSI", item_number: "2",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "Kematian Kerja: 0 | LTI: 0 | RWC: 0 | MTC: 0 | FAC: 0 | Nearmiss: 0 | Kerusakan Properti: 0 | Kecelakaan Kendaraan: 0 | Kebakaran: 0 | FR: 0,00 | SR: 0,00 | TRIR: 0,00",
            code: "A.2.1",
            description: "Pengarahan singkat keselamatan kerja (Safety Briefing / P5M) sebelum pengujian laboratorium & keberangkatan inspeksi/sampling lapangan",
            frequency: "Setiap hari kerja (242 hari)",
            reference_law: "Kepka BPOM 2021; SK Tim P2K3 BBPOM Bandung 2026",
            pic: "Seluruh Unit Kerja & Pengelola Lab",
            output_evidence: "Logbook P5M Lab & Surat Tugas Inspeksi",
            annual_target: 242.0, is_target_auto_sum: true,
            months: [("19",19.0),("19",19.0),("17",17.0),("22",22.0),("17",17.0),("21",21.0),("22",22.0),("21",21.0),("22",22.0),("21",21.0),("22",22.0),("19",19.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "I. PENGELOLAAN FAKTOR MANUSIA & KOMPETENSI", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.2",
            description: "Induksi K3 (Safety Induction) bagi pegawai baru/mutasi, mahasiswa PKL/magang, tamu laboratorium, dan pihak ketiga/kontraktor",
            frequency: "Sesuai kedatangan (min. 1x per triwulan)",
            reference_law: "UU 1/1970 Ps. 9; Panduan Keselamatan Lab",
            pic: "Koordinator Keselamatan Kerja & Sekretaris P2K3",
            output_evidence: "Daftar hadir & kartu induksi K3 tamu/PKL",
            annual_target: 4.0, is_target_auto_sum: false,
            months: [("Tentatif",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "I. PENGELOLAAN FAKTOR MANUSIA & KOMPETENSI", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.3",
            description: "Inspeksi kepatuhan penggunaan Alat Pelindung Diri (APD) standar di laboratorium kimia/mikrobiologi dan kegiatan inspeksi lapangan",
            frequency: "1x per bulan",
            reference_law: "Panduan Keselamatan Lab; Kepka BPOM 2021",
            pic: "Koordinator Keselamatan Kerja (Huruf E.13)",
            output_evidence: "Checklist inspeksi kepatuhan APD bulanan",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "I. PENGELOLAAN FAKTOR MANUSIA & KOMPETENSI", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.4",
            description: "Pendidikan, pelatihan, dan penyegaran K3, SMK3, HIRADC, keselamatan kimia/biorisiko, serta P3K bagi pegawai dan anggota P2K3",
            frequency: "1x per triwulan",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.15; Modul SMK3 BPOM 2026",
            pic: "Koordinator Keselamatan Kerja (Huruf E.15)",
            output_evidence: "Sertifikat/daftar hadir & materi pelatihan",
            annual_target: 4.0, is_target_auto_sum: false,
            months: [("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "II. PENGELOLAAN FAKTOR PERALATAN, SARANA PRASARANA & TEMPAT KERJA", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.5",
            description: "Inspeksi, pengujian fungsi, dan pemeliharaan APAR, APAB, hidran, sprinkler, alarm kebakaran, serta perlindungan radiasi ruang alat",
            frequency: "1x per 6 bulan",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.11 & E.14; Kepka BPOM 2021",
            pic: "Koordinator Keselamatan Kerja (Huruf E.11 & E.14)",
            output_evidence: "Kartu gantung APAR & berita acara inspeksi",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "II. PENGELOLAAN FAKTOR PERALATAN, SARANA PRASARANA & TEMPAT KERJA", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.6",
            description: "Inspeksi isi kotak P3K, pancuran darurat (safety shower), dan stasiun cuci mata (eyewash) laboratorium (radius bebas hambatan 1 m)",
            frequency: "1x per bulan",
            reference_law: "Panduan Keselamatan Lab; Kepka BPOM 2021",
            pic: "Koordinator Keselamatan Kerja & Koordinator Kesehatan Kerja",
            output_evidence: "Log uji aliran air mingguan & rekap bulanan",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "II. PENGELOLAAN FAKTOR PERALATAN, SARANA PRASARANA & TEMPAT KERJA", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.7",
            description: "Audit aksesibilitas fasilitas, rambu K3 visual-taktil, jalur evakuasi, dan titik kumpul inklusif difabel di kantor & laboratorium",
            frequency: "1x per 6 bulan",
            reference_law: "Materi K3 Inklusif SIGAB; Kepka BPOM 2021",
            pic: "Koordinator Keselamatan Kerja & Perwakilan Pegawai",
            output_evidence: "Laporan audit aksesibilitas & foto dokumentasi",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "II. PENGELOLAAN FAKTOR PERALATAN, SARANA PRASARANA & TEMPAT KERJA", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.8",
            description: "Inspeksi kerapian area kerja (5R/housekeeping), lemari asam (fume hood), biosafety cabinet (BSC), panel listrik (radius bebas 1 m), GFCI area basah, dan penyimpanan bahan kimia inkompatibel/peroksida",
            frequency: "2x per bulan",
            reference_law: "Panduan Keselamatan Lab; SNI ISO 35001:2019; SK Tim P2K3 2026",
            pic: "Koordinator Keselamatan Kerja & Pengelola Lab",
            output_evidence: "Formulir inspeksi K3 laboratorium dwimingguan",
            annual_target: 24.0, is_target_auto_sum: true,
            months: [("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0),("2",2.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "III. PENGELOLAAN FAKTOR METODE KERJA & TANGGAP DARURAT", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.9",
            description: "Inventarisasi masalah K3, identifikasi potensi bahaya, dan pemutakhiran dokumen HIRADC (Initial & Residual Risk) mencakup 6 jenis risiko K3 inklusif difabel di perkantoran, lab, dan inspeksi lapangan",
            frequency: "1x per tahun (dan saat ada perubahan proses/alat)",
            reference_law: "Permenaker 13/2025 Ps. 10; Modul SMK3 & HIRADC BPOM 2026; Materi SIGAB",
            pic: "Ketua, Sekretaris, Koordinator K3 & Seluruh Unit",
            output_evidence: "Dokumen Register HIRADC 2027 disahkan Kepala Balai",
            annual_target: 1.0, is_target_auto_sum: false,
            months: [("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "III. PENGELOLAAN FAKTOR METODE KERJA & TANGGAP DARURAT", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.10",
            description: "Reviu dan pemutakhiran SOP keselamatan kerja laboratorium, perkantoran, inspeksi lapangan, izin kerja lembur/berbahaya, serta penyesuaian kerja pegawai hamil",
            frequency: "1x per tahun",
            reference_law: "Kepka BPOM 2021; Panduan Keselamatan Lab; Kepka Sestama 100/2025",
            pic: "Tim P2K3 & Pengelola Lab",
            output_evidence: "Dokumen SOP K3 terkendali dalam SMPOMT",
            annual_target: 1.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "III. PENGELOLAAN FAKTOR METODE KERJA & TANGGAP DARURAT", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.11",
            description: "Simulasi (drill) tanggap darurat kebakaran, gempa bumi, dan tumpahan bahan kimia B3 yang melibatkan pegawai difabel serta evaluasi sistem peringatan ganda (suara & visual)",
            frequency: "1x per tahun",
            reference_law: "PP 50/2012; Panduan Keselamatan Lab; Materi K3 Inklusif SIGAB",
            pic: "Tim P2K3, Tim Tanggap Darurat & Seluruh Pegawai",
            output_evidence: "Skenario simulasi, dokumentasi & laporan evaluasi",
            annual_target: 1.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "III. PENGELOLAAN FAKTOR METODE KERJA & TANGGAP DARURAT", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.12",
            description: "Pemeriksaan penerapan K3 (kesesuaian pelaksanaan pekerjaan dengan SOP K3) dan pemberian saran perbaikan penerapan K3 di setiap unit kerja",
            frequency: "1x per bulan",
            reference_law: "Permenaker 13/2025 Ps. 10; Kepka BPOM 2021",
            pic: "Koordinator Keselamatan Kerja & Sekretaris P2K3",
            output_evidence: "Checklist pemantauan & lembar saran perbaikan K3",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "IV. PENGELOLAAN SISTEM MANAJEMEN, STATISTIK & AUDIT TERINTEGRASI", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.13",
            description: "Rapat rutin bulanan Tim P2K3, Patroli Keselamatan (Safety Patrol), serta analisa dan statistik data kecelakaan kerja, PAK, dan nearmiss",
            frequency: "1x per bulan",
            reference_law: "Permenaker 13/2025 Ps. 9–10; SK Tim P2K3 BBPOM Bandung 2026",
            pic: "Ketua & Sekretaris P2K3",
            output_evidence: "Undangan, daftar hadir, notulen rapat & rekap statistik",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "IV. PENGELOLAAN SISTEM MANAJEMEN, STATISTIK & AUDIT TERINTEGRASI", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.14",
            description: "Sosialisasi Kebijakan K3 BPOM, Manual Sistem Manajemen Terintegrasi (SMPOMT), serta alur pelaporan insiden/kecelakaan (maks. 2x24 jam ke Disnaker & internal)",
            frequency: "1x per 6 bulan",
            reference_law: "Permenaker 03/Men/1998; Kepka BPOM 2021; Kepka BPOM 289/2026",
            pic: "Ketua & Sekretaris P2K3",
            output_evidence: "Materi sosialisasi, daftar hadir & poster alur lapor",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "A", pillar_title: PILAR_A, subcategory: "IV. PENGELOLAAN SISTEM MANAJEMEN, STATISTIK & AUDIT TERINTEGRASI", item_number: "",
            objective: "Pencegahan Kecelakaan Kerja dengan Target Nihil Kecelakaan (Zero Accident)",
            measurable_target: "",
            code: "A.2.15",
            description: "Audit Internal SMK3 / Sistem Manajemen Pengawasan Obat dan Makanan Terintegrasi (SMPOMT) dan Rapat Tinjauan Manajemen (RTM) K3L",
            frequency: "1x per tahun",
            reference_law: "PP 50/2012; ISO 45001:2018; Kepka BPOM 290/2026",
            pic: "Kepala BBPOM, Tim Auditor Internal & Ketua P2K3",
            output_evidence: "Laporan Audit Internal & Risalah RTM",
            annual_target: 1.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Audit",0.5),("RTM",0.5),("",0.0)],
        },
        // PILAR B
        SeedProgram {
            pillar: "B", pillar_title: PILAR_B, subcategory: "", item_number: "1",
            objective: "Pencegahan Penyakit Akibat Kerja (PAK) dan Peningkatan Derajat Kesehatan Pegawai",
            measurable_target: "Penyakit Akibat Kerja (PAK): 0\nCakupan MCU Pegawai: 100%\nPemenuhan NAB Lingkungan Kerja: 100%",
            code: "B.1.1",
            description: "Program Pemeriksaan Kesehatan Berkala (MCU), pemeriksaan kesehatan sebelum bekerja (prakerja), dan pemeriksaan khusus petugas laboratorium",
            frequency: "1x per tahun",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf D.1–2; Permenaker 02/Men/1980; Kepka BPOM 2021",
            pic: "Koordinator Kesehatan Kerja (Huruf D.1–2)",
            output_evidence: "Rekapitulasi hasil MCU & tindak lanjut medis",
            annual_target: 1.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "B", pillar_title: PILAR_B, subcategory: "", item_number: "",
            objective: "Pencegahan Penyakit Akibat Kerja (PAK) dan Peningkatan Derajat Kesehatan Pegawai",
            measurable_target: "",
            code: "B.1.2",
            description: "Pemantauan tekanan darah (tensi), skrining kesehatan dasar, pemberian bantuan kesehatan, dan pencatatan/pelaporan kesehatan pegawai",
            frequency: "1x per bulan",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf D.3–4",
            pic: "Koordinator Kesehatan Kerja (Huruf D.3–4)",
            output_evidence: "Kartu kontrol kesehatan & laporan ke Ketua P2K3",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "B", pillar_title: PILAR_B, subcategory: "", item_number: "",
            objective: "Pencegahan Penyakit Akibat Kerja (PAK) dan Peningkatan Derajat Kesehatan Pegawai",
            measurable_target: "",
            code: "B.1.3",
            description: "Pengujian & pengukuran K3 Lingkungan Kerja (Faktor Fisika, Kimia, Biologi, Ergonomi, dan Psikologi) oleh lembaga/petugas berkompeten",
            frequency: "1x per tahun",
            reference_law: "Permenaker 5/2018; Kepka BPOM 2021 Ps. 15–19; SK Tim P2K3 2026 Huruf E.2",
            pic: "Koordinator Keselamatan Kerja & Koordinator Kesehatan Kerja",
            output_evidence: "Laporan hasil uji lingkungan kerja & tindak lanjut NAB",
            annual_target: 1.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "B", pillar_title: PILAR_B, subcategory: "", item_number: "",
            objective: "Pencegahan Penyakit Akibat Kerja (PAK) dan Peningkatan Derajat Kesehatan Pegawai",
            measurable_target: "",
            code: "B.1.4",
            description: "Pendidikan dan penyuluhan kesehatan kerja, kesehatan lingkungan, gizi kerja, pencegahan penyakit menular, ergonomi, serta kesehatan psikososial",
            frequency: "1x per 6 bulan",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf D.3 & E.12; ISO 45003; ISO 45006",
            pic: "Koordinator Kesehatan Kerja & Koordinator Keselamatan Kerja",
            output_evidence: "Dokumentasi kegiatan & daftar hadir peserta",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0)],
        },
        SeedProgram {
            pillar: "B", pillar_title: PILAR_B, subcategory: "", item_number: "",
            objective: "Pencegahan Penyakit Akibat Kerja (PAK) dan Peningkatan Derajat Kesehatan Pegawai",
            measurable_target: "",
            code: "B.1.5",
            description: "Pembinaan dan pengawasan sanitair kantor & laboratorium: penyehatan air, penyehatan makanan/minuman kantin, tempat pencucian, sterilisasi/desinfeksi, serta pengendalian serangga dan tikus",
            frequency: "1x per triwulan",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.3–10; Permenaker 5/2018",
            pic: "Koordinator Keselamatan Kerja (Huruf E.3–10) & Koordinator Kesehatan Kerja",
            output_evidence: "Checklist sanitasi triwulanan & bukti pest control",
            annual_target: 4.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0)],
        },
        // PILAR C
        SeedProgram {
            pillar: "C", pillar_title: PILAR_C, subcategory: "", item_number: "1",
            objective: "Kepatuhan pada Peraturan Perundang-undangan Lingkungan Hidup",
            measurable_target: "Kepatuhan regulasi lingkungan: 100%\nPelaporan UKL-UPL / Limbah B3 tepat waktu: 100%",
            code: "C.1.1",
            description: "Evaluasi pemenuhan peraturan lingkungan hidup, pemantauan baku mutu air limbah domestik/lab, dan pelaporan pengelolaan Limbah B3 (Green Laboratory)",
            frequency: "1x per 6 bulan",
            reference_law: "UU 32/2009; PP 22/2021; Permen LH/BPLH 11/2025; ISO 14001:2026",
            pic: "Koordinator Keselamatan Kerja, Koordinator Kesehatan Kerja & Pengelola Lab",
            output_evidence: "Laporan semesteran lingkungan & hasil uji IPAL",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "C", pillar_title: PILAR_C, subcategory: "", item_number: "2",
            objective: "Pencegahan Pencemaran Lingkungan & Pengendalian Limbah B3",
            measurable_target: "Pencemaran akibat sampah / tumpahan Limbah B3: 0",
            code: "C.2.1",
            description: "Pengawasan penanganan sampah & limbah: pemilahan, pelabelan simbol B3, pemantauan kapasitas wadah limbah (maks. 75%), pemisahan safety box jarum (kuning) & benda tajam (putih), serta pengangkutan Limbah B3 berizin",
            frequency: "1x per bulan (pengangkutan sesuai masa simpan)",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.8; Panduan Keselamatan Lab; PP 22/2021",
            pic: "Koordinator Keselamatan Kerja (Huruf E.8), Pengelola Lab & Koordinator Kesehatan Kerja",
            output_evidence: "Neraca Limbah B3 bulanan & manifest elektronik (Festronik)",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
        // PILAR D
        SeedProgram {
            pillar: "D", pillar_title: PILAR_D, subcategory: "", item_number: "1",
            objective: "Penanganan Keluhan dan Kepuasan Layanan K3 yang Aksesibel",
            measurable_target: "Keluhan K3 tidak tertangani: 0\nTingkat penyelesaian tindak lanjut: 100%",
            code: "D.1.1",
            description: "Pemantauan umpan balik layanan K3 dan pengelolaan saluran pengaduan K3/psikososial yang aksesibel bagi seluruh pegawai (termasuk pembaca layar & saluran non-verbal)",
            frequency: "1x per triwulan",
            reference_law: "Kepka BPOM 2021; Materi K3 Inklusif SIGAB; ISO 9001:2015",
            pic: "Sekretaris P2K3",
            output_evidence: "Rekap kotak saran/kanal digital K3 & bukti tindak lanjut",
            annual_target: 4.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "D", pillar_title: PILAR_D, subcategory: "", item_number: "2",
            objective: "Pengendalian Kualitas APD & Perlengkapan K3",
            measurable_target: "Kerusakan/kedaluwarsa perlengkapan K3 saat digunakan: 0",
            code: "D.2.1",
            description: "Pembinaan dan pengawasan perlengkapan keselamatan kerja (APD/P3K/alat darurat) saat penerimaan barang dan uji kesesuaian ukuran/kebutuhan pengguna (termasuk akomodasi layak pegawai difabel)",
            frequency: "1x per 6 bulan (dan setiap pengadaan)",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.13; Materi SIGAB",
            pic: "Koordinator Keselamatan Kerja (Huruf E.13) & Tim Pengadaan",
            output_evidence: "Berita acara pemeriksaan barang & kartu stok APD",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "D", pillar_title: PILAR_D, subcategory: "", item_number: "3",
            objective: "Pemeliharaan Kondisi Gedung & Fasilitas Sanitasi Inklusif",
            measurable_target: "Kerusakan sarana gedung/sanitasi tidak tertangani: 0",
            code: "D.3.1",
            description: "Pembinaan dan pengawasan sarana/prasarana gedung, instalasi listrik, tata udara, serta kelayakan toilet aksesibel dan ruang ganti aman berperspektif gender",
            frequency: "1x per triwulan",
            reference_law: "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.1 & E.4; Materi SIGAB",
            pic: "Koordinator Keselamatan Kerja & Subbagian Tata Usaha",
            output_evidence: "Formulir inspeksi gedung & tiket pemeliharaan",
            annual_target: 4.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("Rencana",1.0)],
        },
        SeedProgram {
            pillar: "D", pillar_title: PILAR_D, subcategory: "", item_number: "4",
            objective: "Mewujudkan Lingkungan Kerja Aman, Inklusif, dan Bebas Diskriminasi",
            measurable_target: "Kasus diskriminasi/perundungan/pelanggaran keamanan: 0",
            code: "D.4.1",
            description: "Sosialisasi lingkungan kerja inklusif difabel (prinsip Nothing About Us Without Us), ruang aman, anti-perundungan, dan akomodasi yang layak",
            frequency: "1x per 6 bulan",
            reference_law: "Materi K3 Inklusif SIGAB; ISO 45003:2021",
            pic: "Ketua, Sekretaris & Seluruh Anggota P2K3",
            output_evidence: "Dokumentasi sosialisasi & survei iklim kerja inklusif",
            annual_target: 2.0, is_target_auto_sum: false,
            months: [("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0),("",0.0),("",0.0),("",0.0),("Rencana",1.0),("",0.0),("",0.0)],
        },
        SeedProgram {
            pillar: "D", pillar_title: PILAR_D, subcategory: "", item_number: "",
            objective: "Mewujudkan Lingkungan Kerja Aman, Inklusif, dan Bebas Diskriminasi",
            measurable_target: "",
            code: "D.4.2",
            description: "Pengamanan gedung, kontrol akses laboratorium terbatas, pemeriksaan tamu/kendaraan, dan patroli harian Satpam serta pelaporan keamanan 6 bulanan",
            frequency: "Setiap bulan (rekap patroli harian)",
            reference_law: "Kepka BPOM HK.02.02.1.2.06.21.251/2021 Ps. 25–33",
            pic: "Koordinator Keamanan / Komandan Regu Satpam",
            output_evidence: "Logbook patroli harian Satpam & laporan 6 bulanan",
            annual_target: 12.0, is_target_auto_sum: false,
            months: [("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0),("Rencana",1.0)],
        },
    ];

    for (idx, p) in programs.iter().enumerate() {
        let prog_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO work_programs (
                id, period_id, pillar, pillar_title, subcategory, item_number, objective,
                measurable_target, code, description, frequency, reference_law, pic,
                output_evidence, annual_target, is_target_auto_sum, sort_order
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)",
        )
        .bind(prog_id)
        .bind(period_id)
        .bind(p.pillar)
        .bind(p.pillar_title)
        .bind(p.subcategory)
        .bind(p.item_number)
        .bind(p.objective)
        .bind(p.measurable_target)
        .bind(p.code)
        .bind(p.description)
        .bind(p.frequency)
        .bind(p.reference_law)
        .bind(p.pic)
        .bind(p.output_evidence)
        .bind(p.annual_target)
        .bind(p.is_target_auto_sum)
        .bind((idx + 1) as i32)
        .execute(pool)
        .await?;

        for (m_idx, (label, val)) in p.months.iter().enumerate() {
            sqlx::query(
                "INSERT INTO work_program_monthly (program_id, month, plan_label, plan_value, actual_value)
                 VALUES ($1, $2, $3, $4, 0.0)",
            )
            .bind(prog_id)
            .bind((m_idx + 1) as i32)
            .bind(label)
            .bind(val)
            .execute(pool)
            .await?;
        }
    }

    // 5. Seed 15 Lagging Indicators (Bagian E)
    let lagging_items = [
        (1, "Kematian Akibat Kerja (Fatality)", "fatality", "Bulanan (rekap harian)", "Permenaker 01/Men/2007 & Catatan Insiden K3", "Sekretaris P2K3 & Seluruh Unit", "Target Nihil (0)", 0.0, false),
        (2, "Cedera Hilang Hari Kerja / Lost Time Injury (LTI / LWDC)", "lti", "Bulanan (rekap harian)", "Kepdirjen Binawas Kep. 84/BW/1998", "Sekretaris P2K3 & Seluruh Unit", "Target Nihil (0)", 0.0, false),
        (3, "Kasus Kerja Terbatas (Restricted Work Case / RWC)", "rwc", "Bulanan (rekap harian)", "Catatan Insiden K3 BBPOM Bandung", "Sekretaris P2K3 & Seluruh Unit", "Target Nihil (0)", 0.0, false),
        (4, "Kasus Perawatan Medis (Medical Treatment Case / MTC)", "mtc", "Bulanan (rekap harian)", "Catatan Insiden K3 BBPOM Bandung", "Sekretaris P2K3 & Seluruh Unit", "Target Nihil (0)", 0.0, false),
        (5, "Kasus Pertolongan Pertama (First Aid Case / FAC)", "fac", "Bulanan (rekap harian)", "Catatan Insiden K3 BBPOM Bandung", "Koordinator Kesehatan Kerja", "Target Nihil (0)", 0.0, false),
        (6, "Penyakit Akibat Kerja (PAK)", "pak", "Bulanan", "Perpres 7/2019 & Permenaker 01/Men/2007", "Koordinator Kesehatan Kerja", "Target Nihil (0)", 0.0, false),
        (7, "Kejadian Nyaris Celaka (Nearmiss)", "nearmiss", "Bulanan (rekap harian)", "Modul SMK3 & HIRADC BPOM 2026", "Sekretaris P2K3 & Seluruh Unit", "Wajib dicatat & diinvestigasi", 0.0, false),
        (8, "Kerusakan Properti / Aset Negara (BMN) akibat Insiden", "property_damage", "Bulanan", "Catatan Insiden K3 & Subbag TU", "Sekretaris P2K3 & Subbag TU", "Target Nihil (0)", 0.0, false),
        (9, "Kecelakaan Kendaraan Dinas (Tugas Inspeksi/Sampling)", "vehicle_accident", "Bulanan", "Catatan Insiden K3 & Subbag TU", "Subbag TU & Pengemudi", "Target Nihil (0)", 0.0, false),
        (10, "Insiden Kebakaran", "fire_incident", "Bulanan", "SK Tim P2K3 BBPOM Bandung 2026 Huruf E.14", "Koordinator Keselamatan Kerja", "Target Nihil (0)", 0.0, false),
        (11, "Pencemaran Lingkungan / Tumpahan Bahan Kimia B3", "env_pollution", "Bulanan", "PP 22/2021 & Panduan Keselamatan Lab", "Koordinator Keselamatan Kerja & Pengelola Lab", "Target Nihil (0)", 0.0, false),
        (12, "Jam Kerja Aman Kumulatif Tanpa LTI (Safe Manhours)", "safe_manhours", "Bulanan", "Akumulasi Jam Kerja Efektif Tabel 1", "Sekretaris P2K3", "Link ke Baris Total Tabel 1", 0.0, true),
        (13, "Tingkat Kekerapan Cedera / Frequency Rate (FR)", "fr", "Bulanan", "(Jumlah LTI × 1.000.000) / Total Jam Kerja", "Sekretaris P2K3", "Kepdirjen Binawas 84/1998", 0.0, true),
        (14, "Tingkat Keparahan Cedera / Severity Rate (SR)", "sr", "Bulanan", "(Jumlah Hari Hilang × 1.000.000) / Total Jam Kerja", "Sekretaris P2K3", "Kepdirjen Binawas 84/1998", 0.0, true),
        (15, "Total Tingkat Cedera Tercatat / Total Recordable Incident Rate (TRIR)", "trir", "Bulanan", "((Fatality + LTI + RWC + MTC) × 200.000) / Total Jam Kerja", "Sekretaris P2K3", "Target TRIR = 0,00", 0.0, true),
    ];

    for (num, name, key, freq, ref_f, pic, notes, max_t, is_auto) in lagging_items {
        let ind_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO lagging_indicators (
                id, period_id, item_number, parameter_name, indicator_key, frequency,
                reference_formula, pic, notes, max_target, is_auto_calculated
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
        )
        .bind(ind_id)
        .bind(period_id)
        .bind(num)
        .bind(name)
        .bind(key)
        .bind(freq)
        .bind(ref_f)
        .bind(pic)
        .bind(notes)
        .bind(max_t)
        .bind(is_auto)
        .execute(pool)
        .await?;

        for m in 1..=12 {
            sqlx::query(
                "INSERT INTO lagging_monthly (indicator_id, month, value, lost_days) VALUES ($1, $2, 0.0, 0)",
            )
            .bind(ind_id)
            .bind(m)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
