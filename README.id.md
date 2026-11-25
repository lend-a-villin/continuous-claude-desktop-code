# Continuous Claude Desktop Code (CCDC)

[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | [简体中文](README.zh-Hans.md) | [繁體中文](README.zh-Hant.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md) | [Português](README.pt.md) | [Русский](README.ru.md) | [Українська](README.uk.md) | [Polski](README.pl.md) | [Nederlands](README.nl.md) | [Türkçe](README.tr.md) | [Tiếng Việt](README.vi.md) | **Bahasa Indonesia** | [ไทย](README.th.md) | [العربية](README.ar.md) | [हिन्दी](README.hi.md)

Aplikasi baki sistem Windows yang menjaga sesi **Code** lokal di aplikasi Claude desktop tetap tersedia bagi akun Claude mana pun yang masuk di PC tersebut.

Tidak resmi; tidak berafiliasi dengan dan tidak didukung oleh Anthropic.

## Untuk siapa

CCDC dibuat untuk **tempat kerja yang beroperasi 24 jam dalam tiga shift**. Di setiap shift, orang yang berbeda duduk di meja yang sama (satu PC, satu pengguna Windows) dan masuk ke aplikasi Claude desktop dengan akun Claude masing-masing. Dengan CCDC, petugas shift berikutnya cukup masuk lalu melanjutkan pekerjaan Code yang ditinggalkan shift sebelumnya.

Artinya, setiap sesi Code di PC itu terlihat oleh semua orang yang masuk di sana. Jangan gunakan CCDC di tempat yang penggunanya tidak boleh saling melihat sesi. Untuk akun perusahaan (organisasi), periksa juga kebijakan perusahaan Anda.

## Masalah yang diatasi

Aplikasi desktop menyimpan **daftar** sesi Code di folder terpisah untuk setiap akun dan organisasi:

```
%LOCALAPPDATA%\Claude-Data\claude_<pkg>\claude-code-sessions\<accountUuid>\<orgUuid>\local_<id>.json
```

**Percakapan** itu sendiri ada di `~/.claude/projects\...\<cliSessionId>.jsonl`, yang dipakai bersama oleh semua akun. Karena itu, setelah seseorang masuk dengan akun lain, daftar sesi Code tampak kosong meskipun percakapannya masih ada di disk.

## Cara kerja

Aplikasi membaca folder sesinya **hanya saat dimulai dan saat seseorang masuk** (dua kali, dalam sekitar dua detik). Tidak ada cara untuk membuatnya memuat ulang daftarnya saja di lain waktu. Log aplikasi sendiri, `%LOCALAPPDATA%\Claude\Logs\main.log`, mencatat kedua peristiwa tersebut:

- keluar akun: `[account] Login-state transition (loggedOut: false → true ...)`
- masuk akun: `[LocalSessionManager] Initialization succeeded — accountId=…, orgId=…`

CCDC mengikuti log itu dan bekerja seperti ini:

1. **Keluar akun** → CCDC menyalin versi terbaru setiap sesi ke folder semua akun/organisasi, sehingga akun mana pun yang masuk berikutnya sudah menemukan sesi-sesi itu di foldernya sendiri. Hanya sesi yang belum ada atau sudah usang di suatu folder yang disalin.
2. **Masuk akun** → secara bawaan salinan dibiarkan di tempatnya. Jika "Hapus salinan setelah masuk" diaktifkan, CCDC menunggu sampai aplikasi selesai membaca daftarnya (6 detik setelah baris masuk terakhir), lalu menghapus salinan di semua tempat kecuali folder akun yang sedang masuk.

Sesi yang dihapus di aplikasi juga dihapus dari folder lain, sehingga tidak muncul kembali di akun lain.

Sebelum menyalin, CCDC memeriksa ruang disk kosong. Jika tidak ada ruang untuk salinan ditambah 512 MB, CCDC tidak menyalin apa pun dan menampilkan peringatan.

### Mengapa salinan dipertahankan secara bawaan

Mempertahankan salinan adalah pilihan yang paling sedikit membebani SSD:

- **Salinan dipertahankan (bawaan)**: keluar akun pertama menyalin semua sesi satu kali. Setelah itu, setiap keluar akun hanya menulis sesi yang berubah sejak keluar akun sebelumnya, biasanya beberapa file dan beberapa MB, dan selesai hampir seketika.
- **Salinan dihapus setelah masuk**: setiap masuk akun menghapus salinan, sehingga setiap keluar akun harus menulis ulang semua sesi. Dengan sekitar 300 sesi dan 10 folder akun/organisasi, itu kira-kira 600 MB yang ditulis pada setiap pergantian shift, ditambah penghapusan sekitar 3.000 file sesudahnya.

Sebagai gantinya, salinan memakan ruang disk: salinan tetap ada di disk dan menggunakan sekitar (total ukuran file sesi × jumlah folder akun/organisasi).

### Mengapa tidak memakai junction atau tautan keras

Mengarahkan semua folder akun ke satu folder asli akan menghindari penyalinan, tetapi aplikasi Claude sengaja memblokir kedua cara tersebut:

- **Junction dan tautan simbolis**: sebelum setiap penyimpanan, aplikasi memeriksa apakah folder sesi adalah folder asli dan menolak menyimpan jika berupa tautan (`Refusing non-directory at private dir path (symlink/file plant)`). Dalam uji coba dengan aplikasi sungguhan, daftar bisa dibaca melalui junction, tetapi setiap penyimpanan sesi dan tugas terjadwal gagal, termasuk penyimpanan untuk sesi yang baru dibuat.
- **Tautan keras**: aplikasi menganggap file dengan lebih dari satu tautan tidak aman. Aplikasi menolak menulis ke file seperti itu dan memberi peringatan saat membacanya. Aplikasi juga menyimpan dengan menulis file sementara lalu mengganti namanya, yang akan memutus tautan pada setiap penyimpanan.

Pemeriksaan ini melindungi dari file atau folder yang ditanam dari luar, jadi CCDC tidak mengakalinya. CCDC menyalin file biasa, jenis yang sama dengan yang ditulis aplikasi itu sendiri.

## Persyaratan

- Windows 10 atau 11, dengan aplikasi Claude desktop (instalasi MSIX)
- [Node.js](https://nodejs.org) 18 atau lebih baru

## Instalasi

Unduh penginstal dari [Releases](../../releases), ekstrak, lalu jalankan `setup.cmd`. Tidak perlu hak administrator.

- Diinstal ke `%LOCALAPPDATA%\Programs\CCDC`.
- Menambahkan pintasan `Continuous Claude Desktop Code` di desktop dan menu Start.
- Muncul di Pengaturan > Aplikasi > Aplikasi terinstal sebagai "Continuous Claude Desktop Code (CCDC)"; hapus instalannya dari sana.
- Langsung berjalan di baki sistem setelah diinstal.
- Agar CCDC berjalan otomatis saat Anda masuk ke Windows, instal dengan `setup.cmd /startup`; untuk menonaktifkannya, gunakan `setup.cmd /nostartup`. Tanpa keduanya, instal ulang mempertahankan pengaturan sebelumnya.
- Instal ulang menghentikan CCDC jika sedang berjalan dan mengganti file-nya; pengaturan Anda (`ccdc.json`) tetap disimpan.
- Menghapus instalan membiarkan file sesi Claude dan folder cadangan CCDC apa adanya.

Untuk menjalankannya langsung dari repositori ini, jalankan `ccdc-tray.vbs`, atau gunakan baris perintah di bawah.

## Aplikasi baki sistem

Ikon **C** oranye muncul di area notifikasi (abu-abu saat sinkronisasi otomatis mati). Klik untuk membuka menu:

| Item menu | Fungsi |
| --- | --- |
| Baris status | Menunjukkan apakah CCDC sedang memantau masuk dan keluar akun |
| Sinkronisasi otomatis (deteksi masuk/keluar akun) | Mengaktifkan atau menonaktifkan penyalinan dan pembersihan |
| Pindahkan sesi ke akun sekarang | Mengumpulkan semua sesi ke folder satu akun; bertanya dulu jika akun itu tidak sedang masuk |
| Hapus salinan setelah masuk | Nonaktif secara bawaan (salinan dipertahankan; lihat di atas) |
| Kecepatan salin/hapus | Cepat (tanpa batas), Normal (60 MB/dtk), atau Pelan (15 MB/dtk). Pada Normal dan Pelan, penghapusan dibatasi masing-masing 500 dan 100 file per detik |
| Baris perkiraan | Menunjukkan berapa lama penyalinan saat keluar akun berikutnya, serta lama pembersihan setelah masuk jika salinan dihapus |
| Language | Memilih bahasa tampilan. Item ini selalu berlabel bahasa Inggris; setiap bahasa di submenu ditulis dalam bahasa itu sendiri |
| Lihat log | Membuka `%LOCALAPPDATA%\CCDC\watch.log` |
| Beri nama akun (ccdc.json) | Membuka file pengaturan untuk memberi nama yang mudah dibaca pada akun |
| Keluar | Menghapus ikon; jika penyalinan sedang berjalan, menunggu sampai selesai |

Notifikasi muncul saat penyalinan dimulai (beserta perkiraan waktunya), saat selesai, saat masuk akun terdeteksi, dan saat pembersihan selesai. Dengan kecepatan yang lebih pelan, penyalinan memakan waktu lebih lama, jadi orang berikutnya sebaiknya menunggu notifikasi "Penyalinan selesai", atau kira-kira selama waktu perkiraan, sebelum masuk.

Perkiraan dihitung dari keadaan folder saat ini dan dari kecepatan salin dan hapus yang diukur di PC ini (disimpan di `%LOCALAPPDATA%\CCDC\state.json`).

Jika ikon tidak terlihat di Windows 11, periksa ikon tersembunyi (`^`) di bilah tugas. Agar selalu terlihat, aktifkan Windows PowerShell di Pengaturan > Personalisasi > Bilah tugas > Ikon baki sistem lainnya.

### Penggunaan memori

Aplikasi baki memantau sendiri log Claude dan hanya membaca bagian yang ditambahkan sejak pemeriksaan terakhir. Node.js hanya berjalan saat menyalin atau membersihkan, lalu berhenti, sehingga di antara pergantian shift hanya aplikasi baki yang berjalan. Saat menganggur, aplikasi baki mengembalikan memori yang tidak terpakai ke Windows kira-kira setiap menit (sekitar 15 MB terpakai).

## Pengaturan

Pengaturan disimpan di `ccdc.json` di folder program. Perubahan yang Anda buat di menu baki disimpan ke file ini dan berlaku mulai penyalinan atau pembersihan berikutnya, tanpa perlu memulai ulang.

```json
{
  "keepCopies": true,
  "speed": "fast",
  "language": "",
  "aliases": { "<first 8 characters of an account ID>": "<name>" }
}
```

| Kunci | Nilai |
| --- | --- |
| `keepCopies` | `true` (bawaan): salinan dipertahankan. `false`: salinan dihapus setelah masuk |
| `speed` | `"fast"` (bawaan), `"normal"`, `"gentle"` |
| `language` | Kosong untuk mengikuti bahasa Windows, atau kode bahasa seperti `"ja"` untuk memakai bahasa itu |
| `aliases` | Nama yang ditampilkan untuk akun di menu baki |

## Bahasa

Menu baki, notifikasi, peringatan, dan jendela instal/hapus instalan mengikuti bahasa tampilan Windows, atau bahasa yang dipilih di **Language** pada menu baki. Didukung: Inggris, Korea, Jepang, Tionghoa (Sederhana dan Tradisional), Spanyol, Prancis, Jerman, Italia, Portugis, Rusia, Ukraina, Polandia, Belanda, Turki, Vietnam, Indonesia, Thai, Arab, dan Hindi. Bahasa lain ditampilkan dalam bahasa Inggris.

Semua teks antarmuka ada dalam satu lembar, [`locales.csv`](locales.csv): **satu baris untuk satu bahasa**, satu kolom untuk satu pesan. Buka dengan Excel atau Google Spreadsheet; untuk menambah bahasa, tambahkan satu baris.

- `lang`: nama budaya Windows (`sv`, `pt-BR`, `zh-Hant`, …). Baris yang cocok dengan bahasa dan wilayah diutamakan, lalu baris yang cocok dengan bahasanya saja.
- `name`: nama bahasa itu dalam bahasanya sendiri (`Svenska`); menu Language menampilkannya.
- Tulis baris baru di dalam sel sebagai `\n`, dan pertahankan `{0}`, `{1}` (angka atau nama dimasukkan di situ).
- Sel kosong ditampilkan dalam bahasa Inggris. Simpan file sebagai CSV UTF-8.

Keluaran baris perintah dan `watch.log` tetap dalam bahasa Inggris karena aplikasi baki membacanya.

## Baris perintah

```powershell
node ccdc.mjs status               # app state, settings, sessions per folder, free disk
node ccdc.mjs estimate             # how long the next sign-out copy / sign-in cleanup would take
node ccdc.mjs replicate --dry-run  # what a sign-out copy would write right now
node ccdc.mjs move [--to <acct>]   # gather every session into one account's folder (first 8 characters of the ID are enough)
node ccdc.mjs watch                # follow the app log in the foreground (the tray does this itself)
```

Bila perlu, aplikasi baki menjalankan tugas sekali jalan berikut: `node ccdc.mjs signout`, `node ccdc.mjs signin --account <id> --org <id>`, dan `node ccdc.mjs startup` (untuk menyesuaikan diri dengan keadaan Claude saat ini ketika aplikasi baki dimulai atau sinkronisasi otomatis dinyalakan kembali).

## Aturan untuk duplikat

- Jika satu sesi ada di beberapa folder, salinan dengan `lastActivityAt` terbaru yang dipakai. Salinan mempertahankan waktu modifikasi sumbernya, sehingga salinan yang identik dikenali dari ukuran dan waktu tanpa membacanya.
- Saat salinan yang lebih baru menggantikan sebuah file, file lama dipindahkan ke `ccdc-backups\<waktu>\` di folder data aplikasi.
- Sesi dengan penanda `deleted_<id>` tidak disalin, dan salinannya di folder lain dihapus.
- Akun/organisasi yang masuk untuk pertama kalinya belum memiliki folder, jadi saat keluar akun tidak ada yang bisa disalin ke sana. Dalam hal ini, sesi akan muncul setelah akun itu masuk lagi atau setelah aplikasi dimulai ulang.

## Catatan

- CCDC bergantung pada struktur file internal dan redaksi log aplikasi Claude desktop; pembaruan aplikasi dapat mengubah perilakunya.
- File sesi menyimpan pengaturan konektor (MCP) per akun, sehingga beberapa konektor mungkin tidak tersambung saat sesi dibuka dari akun lain.
- Tugas terjadwal tidak dibagikan; setiap folder akun/organisasi menyimpan tugasnya sendiri.
- Pergantian akun di Claude Code CLI tidak ditangani. CLI memang sudah bisa membuka sesi apa pun di `~/.claude/projects` dengan `--resume`, apa pun akunnya.

## Lisensi

[MIT](LICENSE)
