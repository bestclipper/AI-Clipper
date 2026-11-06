# AI Clipper · Phase 2

Kostenlose lokale macOS-App für Apple Silicon, mit Tauri 2, React, TypeScript und **mitgeliefertem FFmpeg/FFprobe**. Zielgerät: MacBook Air M1 mit 8 GB RAM, macOS 12 oder neuer.

## Was funktioniert?

- Lokaler Videoimport und Player mit Dauer und Auflösung.
- Ein Testclip mit frei wählbarem Start und **61–90 Sekunden** Länge.
- Automatischer **mittiger 9:16-Bildausschnitt**, skaliert auf **1080 × 1920 Pixel**, 30 fps.
- MP4 mit **H.264 und AAC**, Originalton bleibt erhalten; eine stumme Quelle bleibt stumm.
- Fortschrittsanzeige, Abbruch und automatische Prüfung vor der Erfolgsmeldung.

**Später:** automatische Auswahl der fünf besten Szenen, intelligentes Reframing, englische Untertitel, fünf Exporte in einem Durchlauf und Link-Import. In Phase 2 wählst du zum Testen einen einzelnen Abschnitt aus. Der Button „5 Clips erstellen“ bleibt deaktiviert.

## Installieren – ohne Terminal

1. Öffne [GitHub Actions](https://github.com/bestclipper/AI-Clipper/actions) und den neuesten erfolgreichen Lauf von **Validate and build macOS ARM64** (auch der neue Phase-2-Pull-Request hat eigene Läufe).
2. Lade **AI-Clipper-macOS-ARM64** unter **Artifacts** herunter. GitHub-Anmeldung erforderlich; Downloads bleiben 14 Tage verfügbar.
3. Entpacke das ZIP im Finder, öffne die DMG und ziehe **AI Clipper** nach **Programme**. Beende eine ältere Version, bevor du sie ersetzt.
4. Starte AI Clipper. Du brauchst **keine separate FFmpeg-, Homebrew-, Node- oder Rust-Installation**.

Die App ist ad-hoc signiert, aber ohne Apple Developer ID und nicht notarisiert. macOS kann den ersten Start blockieren. Nutze nach einem Startversuch gegebenenfalls **Systemeinstellungen → Datenschutz & Sicherheit → Dennoch öffnen**. Auf verwalteten Macs kann das gesperrt sein. Gatekeeper muss nicht pauschal deaktiviert werden.

## Einen Testclip erstellen

1. **Video auswählen**: lokale MP4-, MOV-, M4V- oder WebM-Datei importieren. Für die Vorschau wird MP4 mit H.264/AAC empfohlen.
2. Unter **Testclip exportieren** den Start in Sekunden eingeben. Alternativ den Player an die gewünschte Stelle bewegen und **Playerposition übernehmen** klicken.
3. Eine Länge von **61 bis 90 Sekunden** eingeben. Der gesamte Abschnitt muss in der Quelle liegen; kürzere Quellen können nicht exportiert werden.
4. **Abschnitt als MP4 exportieren** klicken und einen lokalen Exportordner wählen. Dialog abbrechen erzeugt keine Datei.
5. Die App zeigt Vorbereitung, Schnitt und abschließende Prüfung. **Erst nach erfolgreicher Prüfung** erscheint die Erfolgsmeldung mit Dateipfad. Der neue Dateiname lautet `AI-Clipper-<eindeutige Nummer>.mp4`; bestehende Dateien und die Originaldatei werden nicht überschrieben.

**Abbrechen** beendet den aktiven Prozess und löscht die unvollständige Ausgabe. Während des Exports sind Dateiwechsel und Zeitauswahl gesperrt. Ein Schließ-/Beenden-Versuch bricht zunächst den Export ab und hält die App für die Bereinigung offen; danach lässt sie sich schließen. Erzwungenes Beenden oder Stromausfall kann eine unvollständige Datei hinterlassen – sie wurde dann nicht als erfolgreich bestätigt.

Ein mittiger Beschnitt entfernt seitliche Bildbereiche (z. B. Chat oder Facecam); es gibt noch keine Gesichtserkennung. Die Vorschau zeigt weiterhin das Original, nicht den beschnittenen Export. Originalton bedeutet, dass die erste Tonspur inhaltlich übernommen und neu als AAC mit 192 kbit/s und 48 kHz codiert wird, keine bitgenaue Audiokopie. Falls die Tonspur vor dem Clipende endet, wird der Rest mit Stille aufgefüllt. Weitere Audio-/Untertitelspuren und Kapitel werden nicht exportiert.

## Automatische Exportprüfung

Die App liest Quelle und Ergebnis mit FFprobe. Sie prüft Dauer, Videoauflösung 1080 × 1920, quadratische Pixel, H.264, AAC bei vorhandener Quelle mit Ton und erhaltene Tonspur. Die Videodauer muss innerhalb von 0,1 Sekunden der Auswahl liegen; für die AAC-Spur gilt eine Toleranz von 0,15 Sekunden. Danach dekodiert FFmpeg den **ganzen exportierten Clip** mit Fehlerabbruch. Bei Fehler oder Abbruch wird die neue Datei gelöscht. Die Fortschrittsanzeige erreicht erst danach 100 %. Während der vollständigen Prüfung bleibt sie bei 90 %; diese Phase benötigt zusätzliche Zeit.

## Speicher und lokale Verarbeitung

- Genau **ein Export gleichzeitig**, keine fünf parallelen Encoder.
- Lesen von Festplatte statt vollständigem Laden in JavaScript; keine Base64-Videos.
- Maximal zwei FFmpeg-Codec-Threads und ein Filter-Thread, 30 fps und begrenzte Bitrate (6 Mbit/s Ziel, maximal 8 Mbit/s).
- Apples VideoToolbox-H.264-Encoder nutzt auf dem M1 verfügbare Hardware; die systemseitige Softwarevariante ist erlaubt, falls Hardware nicht verfügbar ist. Kein zusätzlicher Encoder-Download.
- Begrenzte Prozesslogs; Fortschritt über Tauri-Kanäle. Prüfung erfolgt nach dem Schnitt, nicht parallel.

Ein tatsächlicher RAM-/Langzeitbenchmark auf dem M1 mit 8 GB bleibt nötig. Sehr hoch aufgelöste Quellen oder Codec-Sonderfälle können mehr Ressourcen benötigen. Plane freien Speicher für die Ausgabedatei und FFmpegs Faststart-Umsortierung ein. Ein 90-Sekunden-Clip liegt bei dieser Zielbitrate grob im Bereich von 70 MB; Bildinhalt und Encoder beeinflussen die tatsächliche Größe.

Keine Cloud-API, kein Tracking und keine Videouploads. Die mitgelieferten FFmpeg-Werkzeuge sind ohne Netzwerkprotokolle gebaut. Die App erlaubt als Eingabe nur lokal importierte Dateien; das Backend prüft die Dateifreigabe erneut. FFmpeg erhält Argumente direkt, ohne Shell. Nach einem Neustart muss die Quelle erneut ausgewählt werden.

## Formate und Grenzen

Die Dateiendung garantiert keinen kompatiblen Codec. Der FFmpeg-Build enthält native Decoder für verbreitete H.264-/HEVC-/VP8-/VP9-Videos und gängige Audioformate, aber keine externen Codec-Bibliotheken. System-WebView und FFmpeg können unterschiedliche Formate unterstützen. Nicht unterstützte oder beschädigte Quellen erzeugen eine Fehlermeldung. DRM-geschützte Inhalte werden nicht unterstützt. Es werden keine fremden Untertitel eingebrannt und keine neuen Untertitel erzeugt.

## Kostenloser macOS-ARM64-Build

[Workflow](.github/workflows/build.yml): Frontend-Prüfungen und echte FFmpeg-Integrationstests laufen auf Ubuntu. Der kostenlose Standard-Runner **`macos-15-intel`** baut aus geprüftem, festgeschriebenem FFmpeg-Quellcode die Intel- und ARM64-Werkzeuge. Auf Intel werden die Werkzeuge gestartet und die Filter mit einem Testvideo geprüft; Rust-Tests laufen auf der Hostarchitektur. Tauri erzeugt anschließend die ARM64-App mit `--target aarch64-apple-darwin`.

Vor dem DMG-Upload werden App, FFmpeg und FFprobe auf ARM64 und gültige Ad-hoc-Signaturen geprüft; die DMG wird mit `hdiutil verify` kontrolliert. Die App enthält außerdem FFmpeg-Quellcode und Lizenzhinweise. Ein ARM64-Laufzeittest ist auf dem Intel-Runner nicht möglich.

Das Repository ist öffentlich. [GitHub listet `macos-15-intel` als kostenlosen Standard-Runner](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Es werden keine kostenpflichtigen größeren Runner oder APIs eingesetzt. [Tauri dokumentiert den ARM64-Cross-Build auf Intel](https://tauri.app/distribute/app-store/). Kostenlose Runner können Warteschlangen haben. Bei privaten Repositories gelten Kontingente; ein Ausgabenlimit von 0 verhindert kostenpflichtigen Mehrverbrauch.

## FFmpeg und Lizenzen

Die Anwendung steht unter MIT. FFmpeg n8.0.3 wird als eigenständiges Werkzeug unter LGPL 2.1 oder später mitgeliefert. GPL-/Nonfree-Komponenten und externe Codec-Bibliotheken sind deaktiviert. Quellcode, Lizenztext und exakte Build-Anleitung liegen in der installierten App unter `Contents/Resources/third-party/`. Details: [THIRD_PARTY.md](THIRD_PARTY.md). Es entstehen keine API- oder Lizenzabonnementkosten.

## Entwicklung nur in Codex Cloud

Die folgenden Befehle gehören in die Cloud-Entwicklungsumgebung, **nicht auf deinen Mac**:

```sh
npm ci
npm test
npm run build
npm run test:e2e
cargo test --manifest-path crates/media-core/Cargo.toml --locked
AI_CLIPPER_TEST_FFMPEG=/usr/bin/ffmpeg AI_CLIPPER_TEST_FFPROBE=/usr/bin/ffprobe \
  cargo test --manifest-path crates/media-core/Cargo.toml --locked -- --include-ignored
```

Die echten FFmpeg-Integrationstests sind explizit markiert und werden in CI mit `--include-ignored` verpflichtend ausgeführt. Sie verwenden den Linux-Softwareencoder `libx264` ausschließlich zum Testen; dieser ist nicht in der ausgelieferten App enthalten. Die UI-Vertragstests simulieren die Tauri-Brücke und ersetzen keine native Prüfung.

`npm run dev` bietet eine Browser-Vorschau mit Dateiimport. Ein Export ist dort deaktiviert und als Mac-App-Funktion gekennzeichnet. Für native Entwicklung müssen die jeweiligen Sidecars zuerst in der passenden Cloud-Umgebung gebaut werden; `scripts/build-ffmpeg-macos.sh` übernimmt das auf macOS. Linux kann kein vollständiges macOS-Paket erzeugen.

## Architektur und Prüfbericht

`crates/media-core` enthält die unabhängig von Tauri testbare Prozesssteuerung, den Schnitt und die Exportprüfung. `src-tauri` prüft Dateiberechtigungen, öffnet den Exportordnerdialog und begrenzt gleichzeitige Exporte. `src/ExportPanel.tsx` bietet Auswahl, Fortschritt und Ergebnis. Die spätere automatische Fünf-Clip-Pipeline bleibt in `src/pipeline.ts` vorbereitet.

Tatsächlich ausgeführte Prüfungen und die M1-Abnahmeliste stehen in [VALIDATION.md](VALIDATION.md).
