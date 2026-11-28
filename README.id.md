# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | **Bahasa Indonesia** | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Aplikasi baki sistem yang menjaga sesi **Code** di aplikasi Claude desktop tetap ada saat Anda berganti akun Claude.

Aplikasi Claude desktop menyimpan daftar sesi Code secara terpisah untuk setiap akun dan organisasi. Jika Anda masuk dengan akun lain, daftarnya tampak kosong, padahal percakapannya masih ada di komputer. CCDC menyalin sesi-sesi itu ke semua akun, sehingga akun mana pun yang masuk berikutnya bisa melihat semuanya.

Tidak resmi; tidak berafiliasi dengan dan tidak didukung oleh Anthropic.

**[Unduh rilis terbaru](../../releases/latest)** · Windows 10/11 · macOS 11+ · Ubuntu/Debian/Linux Mint · tidak perlu menginstal apa pun lagi

| Platform | Versi | Status |
| --- | --- | --- |
| Windows | 1.1 | Baru; sudah diuji dengan aplikasi Claude sungguhan di Windows dan berfungsi seperti yang dijelaskan (lanjutan dari skrip 0.3) |
| macOS | 0.2 | Baru; belum diuji dengan aplikasi Claude sungguhan di Mac |
| Linux | 0.2 | Baru; instalasinya sudah diuji di Debian 12, Ubuntu 24.04, dan Linux Mint 22, tetapi belum dengan aplikasi Claude sungguhan |

## Untuk siapa

- **Komputer bersama dan kerja shift.** Di tempat kerja yang beroperasi 24 jam dalam tiga shift, orang yang berbeda duduk di meja yang sama (satu komputer, satu pengguna sistem) di setiap shift dan masuk dengan akun Claude masing-masing. Dengan CCDC, petugas shift berikutnya bisa melanjutkan pekerjaan Code yang ditinggalkan shift sebelumnya.
- **Satu orang, beberapa akun.** Jika Anda berganti antara akun pribadi dan akun kerja, atau antara beberapa organisasi, di komputer yang sama, sesi Code Anda tetap ada dengan akun mana pun Anda masuk.

Setiap sesi Code di komputer itu akan terlihat oleh semua akun yang masuk di sana. Jangan gunakan CCDC di tempat yang penggunanya tidak boleh saling melihat sesi. Jika Anda memakai akun perusahaan (organisasi), periksa dulu kebijakan perusahaan Anda: CCDC juga menyalin sesi akun kerja ke folder akun pribadi, begitu pula sebaliknya.

## Masalah yang diatasi

Aplikasi desktop menyimpan **daftar** sesi Code di folder terpisah untuk setiap akun dan organisasi:

```
<Claude data folder>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
```

| | Folder data Claude | Log Claude |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Claude-Data\claude_<pkg>` (atau `%APPDATA%\Claude`) | `%LOCALAPPDATA%\Claude\Logs\main.log` |
| macOS | `~/Library/Application Support/Claude` | `~/Library/Logs/Claude/main.log` |
| Linux | `~/.config/Claude` | `~/.config/Claude/logs/main.log` |

(Organisasi yang menjalankan Claude melalui penyedia cloud mereka sendiri memakai `Claude-3p`, bukan `Claude`; CCDC juga mencarinya di sana.)

**Percakapan** itu sendiri ada di `~/.claude/projects/.../<cliSessionId>.jsonl`, yang dipakai bersama oleh semua akun. Karena itu, setelah seseorang masuk dengan akun lain, daftar sesi Code tampak kosong meskipun percakapannya masih ada di disk.

## Cara kerja

Aplikasi membaca folder sesinya **hanya saat dimulai dan saat seseorang masuk** (dua kali, dalam sekitar dua detik). Tidak ada cara untuk membuatnya memuat ulang daftarnya saja di lain waktu. Log aplikasi itu sendiri mencatat kedua peristiwa tersebut:

- keluar akun: `[account] Login-state transition (loggedOut: false → true ...)`
- masuk akun: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC mengikuti log itu:

1. **Keluar akun** → CCDC menyalin versi terbaru setiap sesi ke folder semua akun/organisasi, sehingga akun yang masuk berikutnya sudah menemukan sesi-sesi itu di foldernya sendiri. Hanya sesi yang belum ada atau sudah usang di suatu folder yang disalin.
2. **Masuk akun** → secara bawaan salinan dibiarkan di tempatnya. Jika "Hapus salinan setelah masuk" diaktifkan, CCDC menunggu sampai aplikasi selesai membaca daftarnya (6 detik setelah baris masuk terakhir), lalu menghapus salinan di semua tempat kecuali folder akun yang sedang masuk.

Sesi yang dihapus di aplikasi juga dihapus dari folder lain, sehingga tidak muncul kembali di akun lain.

Sebelum menyalin, CCDC memeriksa ruang disk kosong. Jika tidak ada ruang untuk salinan ditambah 512 MB, CCDC tidak menyalin apa pun dan menampilkan peringatan.

### Mengapa salinan dipertahankan secara bawaan

Mempertahankan salinan adalah pilihan yang paling sedikit membebani SSD:

- **Salinan dipertahankan (bawaan)**: keluar akun pertama menyalin semua sesi satu kali. Setelah itu, setiap keluar akun hanya menulis sesi yang berubah sejak keluar akun sebelumnya, biasanya beberapa file dan beberapa MB, dan selesai hampir seketika.
- **Salinan dihapus setelah masuk**: setiap masuk akun menghapus salinan, sehingga setiap keluar akun harus menulis ulang semua sesi. Dengan sekitar 300 sesi dan 10 folder akun/organisasi, itu kira-kira 600 MB yang ditulis pada setiap pergantian shift, ditambah penghapusan sekitar 3.000 file sesudahnya.

Sebagai gantinya, salinan memakan ruang disk: salinan tetap ada di tempatnya dan menggunakan sekitar (total ukuran file sesi × jumlah folder akun/organisasi).

### Mengapa tidak memakai junction atau tautan keras

Mengarahkan semua folder akun ke satu folder asli akan menghindari penyalinan, tetapi aplikasi Claude sengaja memblokir kedua cara tersebut:

- **Junction dan tautan simbolis**: sebelum setiap penyimpanan, aplikasi memeriksa apakah folder sesi adalah folder asli dan menolak menyimpan jika berupa tautan (`Refusing non-directory at private dir path (symlink/file plant)`). Dalam uji coba dengan aplikasi sungguhan, daftar bisa dibaca melalui junction, tetapi setiap penyimpanan sesi dan tugas terjadwal gagal, termasuk penyimpanan untuk sesi yang baru dibuat.
- **Tautan keras**: aplikasi menganggap file dengan lebih dari satu tautan tidak aman. Aplikasi menolak menulis ke file seperti itu dan memberi peringatan saat membacanya. Aplikasi juga menyimpan dengan menulis file sementara lalu mengganti namanya, yang akan memutus tautan pada setiap penyimpanan.

Pemeriksaan ini melindungi dari file atau folder yang ditanam dari luar, jadi CCDC tidak mengakalinya. CCDC menyalin file biasa, jenis yang sama dengan yang ditulis aplikasi itu sendiri.

## Instalasi

Unduh file untuk sistem Anda dari [Releases](../../releases). File-file ini tidak ditandatangani secara digital (lihat [di bawah](#tentang-peringatan-keamanan)).

**Windows** — jalankan `CCDC-<version>-windows-x64.msi`. Tidak perlu hak administrator.

- Diinstal ke `%LOCALAPPDATA%\Programs\CCDC`, menambahkan pintasan `Continuous Claude Desktop Code` di desktop dan menu Start, lalu langsung menjalankan CCDC di baki sistem.
- Berjalan otomatis saat komputer menyala; nonaktifkan dengan "Jalankan saat komputer menyala" di menu baki.
- Memperbarui: jalankan `.msi` yang lebih baru. Penginstal akan menutup CCDC, menggantinya, lalu menjalankannya kembali; pengaturan Anda tetap disimpan.
- Hapus instalan melalui Pengaturan > Aplikasi > Aplikasi terinstal > "Continuous Claude Desktop Code (CCDC)".
- Jika sebelumnya memakai skrip 0.3: cukup instal `.msi`. Saat pertama kali berjalan, CCDC menghentikan versi lama, mempertahankan pengaturan dan nama akunnya, lalu menghapusnya.

**macOS** — buka `CCDC-<version>-macos-universal.dmg`, seret **CCDC** ke Applications, lalu buka aplikasinya. Ikon **C** akan muncul di bilah menu. CCDC berjalan otomatis saat komputer menyala, kecuali Anda menonaktifkannya di menunya. Berjalan di Apple Silicon dan Intel.

**Linux** (Ubuntu 22.04+, Debian 12+, Linux Mint 21+, x86_64):

```sh
sudo apt install ./CCDC-<version>-linux-amd64.deb
```

Setelah itu, jalankan **Continuous Claude Desktop Code** dari menu aplikasi. CCDC berjalan otomatis saat komputer menyala, kecuali Anda menonaktifkannya di menunya. Di GNOME tanpa ekstensi AppIndicator, ikon baki tidak bisa tampil (Ubuntu dan Linux Mint sudah menyertakannya).

Menghapus instalan membiarkan file sesi Claude, pengaturan CCDC, dan folder cadangan tetap di tempatnya.

### Tentang peringatan keamanan

Penandatanganan kode memerlukan sertifikat berbayar yang terikat pada perorangan atau perusahaan, sedangkan CCDC adalah proyek kecil yang gratis, jadi file unduhannya tidak ditandatangani. Sistem Anda akan memberi peringatan saat pertama kali dibuka; artinya hanya penerbitnya belum terverifikasi, bukan karena ditemukan sesuatu yang berbahaya.

CCDC tidak berisi apa pun yang berbahaya. CCDC tidak terhubung ke internet, tidak mengumpulkan data, dan tidak mengubah apa pun di luar folder sesi aplikasi Claude dan foldernya sendiri. Seluruh kodenya ada di repositori ini, dan file rilis dibangun dari kode tersebut oleh GitHub Actions, yang lognya terbuka untuk umum. Anda bisa mencocokkan file unduhan dengan `SHA256SUMS.txt` di halaman rilis, atau [membangunnya sendiri](#membangun).

- **Windows**: jika SmartScreen menampilkan "Windows protected your PC", klik **More info**, lalu **Run anyway**.
- **macOS**: jika macOS mengatakan tidak dapat memverifikasi aplikasi, buka **System Settings > Privacy & Security**, gulir ke bawah, lalu klik **Open Anyway** di samping CCDC. (Atau jalankan `xattr -dr com.apple.quarantine /Applications/CCDC.app` di Terminal.)
- **Linux**: paket `.deb` tidak diperiksa dengan cara ini.

## Aplikasi baki sistem

Ikon **C** oranye muncul di area notifikasi atau bilah menu (abu-abu saat sinkronisasi otomatis mati). Klik untuk membuka menu:

| Item menu | Fungsi |
| --- | --- |
| Baris status | Menunjukkan apakah CCDC sedang memantau masuk dan keluar akun |
| Baris waktu masuk | Satu baris untuk tiap akun yang sedang keluar, berisi berapa lama lalu akun itu masuk (mis. "masuk 2 jam 13 menit lalu"), untuk memperkirakan posisinya dalam jendela penggunaan 5 jam Claude |
| Sinkronisasi otomatis (deteksi masuk/keluar akun) | Mengaktifkan atau menonaktifkan penyalinan dan pembersihan |
| Pindahkan sesi ke akun sekarang | Muncul saat "Hapus salinan setelah masuk" aktif. Untuk saat keluar akun tidak terdeteksi: mengumpulkan semua sesi ke folder satu akun; bertanya dulu jika akun itu tidak sedang masuk |
| Sinkronkan semua akun sekarang | Muncul di tempat yang sama saat "Hapus salinan setelah masuk" nonaktif. Untuk saat keluar akun tidak terdeteksi: menyalin semua sesi ke folder setiap akun, seperti saat keluar akun |
| Hapus salinan setelah masuk | Nonaktif secara bawaan (salinan dipertahankan; lihat di atas) |
| Kecepatan salin/hapus | Cepat (tanpa batas), Normal (60 MB/dtk), atau Pelan (15 MB/dtk). Pada Normal dan Pelan, penghapusan dibatasi masing-masing 500 dan 100 file per detik |
| Baris perkiraan | Menunjukkan berapa lama penyalinan saat keluar akun berikutnya, serta lama pembersihan setelah masuk jika salinan dihapus |
| Jalankan saat komputer menyala | Menjalankan CCDC saat Anda masuk ke komputer |
| Language | Memilih bahasa tampilan. Item ini selalu berlabel bahasa Inggris; setiap bahasa di submenu ditulis dalam bahasa itu sendiri |
| Lihat log | Membuka `watch.log` milik CCDC |
| Beri nama akun (ccdc.json) | Membuka file pengaturan untuk memberi nama yang mudah dibaca pada akun |
| Keluar | Menghapus ikon; jika penyalinan sedang berjalan, menunggu sampai selesai |

Notifikasi muncul saat penyalinan dimulai (beserta perkiraan waktunya), saat selesai, saat masuk akun terdeteksi, dan saat pembersihan selesai. Dengan kecepatan yang lebih pelan, penyalinan memakan waktu lebih lama, jadi orang berikutnya sebaiknya menunggu notifikasi "Penyalinan selesai", atau kira-kira selama waktu perkiraan, sebelum masuk.

Perkiraan dihitung dari keadaan folder saat ini dan dari kecepatan salin dan hapus yang diukur di komputer ini (disimpan di `state.json` di folder CCDC).

Jika ikon tidak terlihat di Windows 11, periksa ikon tersembunyi (`^`) di bilah tugas. Agar selalu terlihat, aktifkan CCDC di Pengaturan > Personalisasi > Bilah tugas > Ikon baki sistem lainnya.

CCDC adalah satu program native berukuran sekitar 1 MB. Selama menunggu, CCDC hanya membaca bagian log Claude yang ditambahkan sejak pemeriksaan terakhir, setiap setengah detik, dan memakai sekitar 4 MB memori untuk dirinya sendiri.

## Pengaturan

Pengaturan disimpan di `ccdc.json` di folder CCDC:

| | Folder CCDC |
| --- | --- |
| Windows | `%LOCALAPPDATA%\CCDC` |
| macOS | `~/Library/Application Support/CCDC` |
| Linux | `~/.config/ccdc` |

Perubahan yang Anda buat di menu baki disimpan ke file ini dan berlaku mulai penyalinan atau pembersihan berikutnya, tanpa perlu memulai ulang. `watch.log` dan `state.json` juga ada di folder yang sama.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "startAtLogin": true,
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Kunci | Nilai |
| --- | --- |
| `keepCopies` | `true` (bawaan): salinan dipertahankan. `false`: salinan dihapus setelah masuk |
| `speed` | `"fast"` (bawaan), `"normal"`, `"gentle"` |
| `language` | Kosong untuk mengikuti bahasa sistem, atau kode bahasa seperti `"ja"` untuk memakai bahasa itu |
| `startAtLogin` | Diatur oleh "Jalankan saat komputer menyala" di menu baki |
| `aliases` | Nama yang ditampilkan untuk akun di menu baki |
| `dataDir`, `appLog` | Hanya jika CCDC tidak bisa menemukan sendiri folder data atau log Claude: path lengkap keduanya |

## Bahasa

Menu baki, notifikasi, dan peringatan mengikuti bahasa tampilan sistem, atau bahasa yang dipilih di **Language** pada menu baki. Didukung: Inggris, Korea, Jepang, Tionghoa (Sederhana dan Tradisional), Spanyol, Prancis, Jerman, Italia, Portugis, Rusia, Ukraina, Polandia, Belanda, Turki, Vietnam, Indonesia, Thai, Arab, dan Hindi. Bahasa lain ditampilkan dalam bahasa Inggris.

Semua teks antarmuka ada dalam satu lembar, [`locales.csv`](locales.csv), yang disertakan ke dalam program: **satu baris untuk satu bahasa**, satu kolom untuk satu pesan. Buka dengan Excel atau Google Spreadsheet; untuk menambah bahasa, tambahkan satu baris lalu [bangun](#membangun) CCDC.

- `lang`: tag bahasa (`sv`, `pt-BR`, `zh-Hant`, …). Baris yang cocok dengan bahasa dan wilayah diutamakan, lalu baris yang cocok dengan bahasanya saja.
- `name`: nama bahasa itu dalam bahasanya sendiri (`Svenska`); menu Language menampilkannya.
- Tulis baris baru di dalam sel sebagai `\n`, dan pertahankan `{0}`, `{1}` (angka atau nama dimasukkan di situ).
- Sel kosong ditampilkan dalam bahasa Inggris. Simpan file sebagai CSV UTF-8.

Keluaran baris perintah dan `watch.log` tetap dalam bahasa Inggris.

## Baris perintah

Program yang sama juga bisa dipakai dari terminal (di Windows, `ccdc.exe` di `%LOCALAPPDATA%\Programs\CCDC`; di macOS, `/Applications/CCDC.app/Contents/MacOS/ccdc`):

```sh
ccdc status               # app state, settings, sessions per folder, last sign-in times, free disk
ccdc estimate             # how long the next sign-out copy / sign-in cleanup would take
ccdc replicate --dry-run  # what a sign-out copy would write right now
ccdc move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
ccdc watch                # follow the Claude log in the foreground, without the tray
ccdc version
```

`--data-dir <path>` dan `--app-log <path>` mengarahkannya secara manual ke folder data atau log Claude.

## Aturan untuk duplikat

- Jika satu sesi ada di beberapa folder, salinan dengan `lastActivityAt` terbaru yang dipakai. Salinan mempertahankan waktu modifikasi sumbernya, sehingga salinan yang identik dikenali dari ukuran dan waktu tanpa membacanya.
- Saat salinan yang lebih baru menggantikan sebuah file, file lama dipindahkan ke `ccdc-backups/<time>/` di folder data Claude.
- Sesi dengan penanda `deleted_<id>` tidak disalin, dan salinannya di folder lain dihapus.
- Akun/organisasi yang masuk untuk pertama kalinya belum memiliki folder, jadi saat keluar akun tidak ada yang bisa disalin ke sana. Dalam hal ini, sesinya akan muncul setelah akun itu masuk lagi atau setelah aplikasi dimulai ulang.

## Catatan

- CCDC bergantung pada struktur file internal dan redaksi log aplikasi Claude desktop; pembaruan aplikasi dapat mengubah perilakunya.
- File sesi menyimpan pengaturan konektor (MCP) per akun, sehingga beberapa konektor mungkin tidak tersambung saat sesi dibuka dari akun lain.
- Tugas terjadwal tidak dibagikan; setiap folder akun/organisasi menyimpan tugasnya sendiri.
- Pergantian akun di Claude Code CLI tidak ditangani. CLI memang sudah bisa membuka sesi apa pun di `~/.claude/projects` dengan `--resume`, apa pun akunnya.

## Membangun

CCDC ditulis dalam Rust; [instal Rust](https://rustup.rs), lalu:

```sh
cargo build --release                                    # the program (target/release/ccdc)
powershell -File packaging\windows\build-msi.ps1         # Windows .msi (needs the .NET SDK)
sh packaging/macos/make-dmg.sh                           # macOS .dmg (on a Mac)
sh packaging/linux/build-deb.sh                          # Linux .deb (on Ubuntu 22.04, or in Docker:)
docker run --rm -v "$PWD:/w" -w /w ubuntu:22.04 sh packaging/linux/build-deb.sh
```

Versi tiap platform ditetapkan di `[package.metadata.ccdc]` dalam `Cargo.toml`. GitHub Actions ([`.github/workflows/build.yml`](.github/workflows/build.yml)) membangun ketiganya pada setiap push dan, untuk tag `v*`, melampirkannya ke rilis draf.

## Lisensi

[MIT](LICENSE)
