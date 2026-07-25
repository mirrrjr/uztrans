# uztrans — Õrnatiş va İşlatiş (Õzbekça)

```
Sh, SH -> Ş     sh -> ş
Ch, CH -> Ç     ch -> ç
G'     -> Ğ     g' -> ğ
O'     -> Õ     o' -> õ
```

uztrans — bu Õzbek lotin digraflarini (sh, ch, g', o') ularning bitta
harfli Unicode kõrinişiga (ş, ç, ğ, õ) õgiradigan buyruq qatori
dasturi, bunda kod, markup va tuzilgan ma'lumotlar õzgarişsiz qoladi.

## 1. Talablar

- Rust va Cargo, versiya 1.75 yoki undan yuqori (barqaror versiya —
  heç qanday beqaror (nightly) xususiyatlar işlatilmagan). Agar Rust
  hali õrnatilmagan bõlsa, https://rustup.rs saytidan õrnating.

## 2. Õrnatiş

Loyihani oçib, uztrans papkasi içida quyidagini bajaring:

```bash
cd uztrans
cargo install --path .
```

Bu optimallaştirilgan dasturni yiğadi va uni `~/.cargo/bin/uztrans`
manziliga nusxalaydi. Õşa papka `PATH` içida ekanligiga işonç hosil
qiling (rustup õrnatuvçisi buni odatda avtomatik qõşadi). Õrnatilgandan
sõng, `uztrans` istalgan papkadan, istalgan terminal seansidan
foydalanişi mumkin bõladi.

Agar butun tizim uçun õrnatişni xohlamasangiz, uni şunçaki yiğib,
loyiha papkasidan işga tuşirişingiz mumkin:

```bash
cargo build --release
./target/release/uztrans --help
```

## 3. Asosiy foydalanış

```bash
# Faylni õzgartirilgan holda terminalga çiqariş
uztrans kitob.md

# Faylni tõğridan-tõğri tahrirlaş (uni qayta yozadi)
uztrans --in-place kitob.md

# Butun papkani, içki papkalar bilan birga qayta işlaş
uztrans --in-place --recursive docs/

# Heç narsa yozmasdan nima õzgarişini kõriş
uztrans --dry-run --diff kitob.md

# Natijani boşqa faylga yoziş, original faylni õzgarişsiz qoldiriş
uztrans kitob.md -o kitob.tarjima.md

# Quvur orqali õqiş va yoziş
cat kitob.md | uztrans > kitob.tarjima.md
```

Barça mavjud bayroqlarni kõriş uçun istalgan vaqtda `uztrans --help`
buyruğini işga tuşiring.

## 4. Nimalarga tegmaydi va nimalarni õzgartiradi

| Fayl turi | Xatti-harakati |
|---|---|
| .md, .markdown | Oddiy matn õzgartiriladi; kod bloklari, içki kod va havola manzillari aynan qoldiriladi. |
| .html, .htm, .xml, .xhtml | Kõrinadigan matn õzgartiriladi; teglar, atributlar, `<script>`/`<style>` içidagi kod va izohlar aynan qoldiriladi. |
| .txt | Butun fayl oddiy matn sifatida qaraladi va õzgartiriladi. |
| boşqa hammasi (.rs, .py, .json va h.k.) | Heç qaçon tegilmaydi, şuning uçun uztrans'ni aralaş fayllar papkasiga qõllaş har doim xavfsiz. |

## 5. Kõp işlatiladigan bayroqlar

| Bayroq | Nima qiladi |
|---|---|
| `-o, --output <PATH>` | Kiritilgan faylni qayta yoziş õrniga boşqa fayl yoki papkaga yozadi. |
| `-i, --in-place` | Kiritilgan fayl(lar)ni tõğridan-tõğri qayta yozadi. |
| `-r, --recursive` | Papka berilganda, uning içki papkalarini ham qayta işlaydi. |
| `--dry-run` | Heç qanday faylni õzgartirmasdan, nima bõlişini kõrsatadi. |
| `--diff` | Õzgarişlarning rangli, qator-qator kõrinişini çiqaradi. |
| `--include <GLOB>` / `--exclude <GLOB>` | Faqat berilgan namunaga mos fayllarni qayta işlaydi (yoki ularni õtkazib yuboradi), masalan `--exclude "*.generated.md"`. |
| `--ext <EXTENSION>` | Standart rõyxatda bõlmagan fayl kengaytmasini ham oddiy matn sifatida belgilaydi. |

Biror narsa tuşunarsiz bõlsa, `uztrans --help` doim sizning
qurilmangizdagi aniq bayroqlar bõyiça eng işonçli manba hisoblanadi.
