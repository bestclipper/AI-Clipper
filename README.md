# AI Clipper · Phase 4

Kostenlose lokale macOS-App für Apple Silicon, mit Tauri 2, React, TypeScript und **mitgeliefertem FFmpeg/FFprobe, whisper.cpp und llama.cpp**. Zielgerät: MacBook Air M1 mit 8 GB RAM, macOS 12 oder neuer.

## Phase 4 · lokale Szenenanalyse

Nach dem Videoimport im Bereich **KI-Analyse · Szenenvorschläge** das kostenlose
Qwen-Modell herunterladen (einmalig 1,28 GB). Whisper base.en muss ebenfalls
installiert sein. **KI-Analyse starten** erstellt bei Bedarf zuerst das englische
Transkript und analysiert dann das gesamte Video. Keine Terminal-Installation,
kein API-Schlüssel und kein Video-Upload. Nach dem Modelldownload offline nutzbar.

- **Single Scene:** ein zusammenhängender Originalabschnitt mit 61–90 Sekunden.
- **Smart Story:** 2–5 chronologische, nicht überlappende Originalabschnitte,
  zusammen 61–90 Sekunden, möglichst nahe 75 Sekunden. Erst Ereignisse in
  überlappenden Fenstern erkennen, dann globale Themen und konkrete Aussagen
  vergleichen. Eine zweite Qwen-Abfrage muss Zusammenhang und Verständlichkeit
  anhand der Originalaussagen begründen. Unbelegte oder unpassende Vorschläge
  werden verworfen; zehn Treffer werden nicht erzwungen.
- Die Liste ist nach Score sortiert. Details enthalten Originalzeiten,
  englischen Titel, Kategorie, Begründung, Belegzitate und Einzelbewertungen.
  **Smart Story als Playlist ansehen** spielt die Teile im Originalvideo ab
  und überspringt die Lücken. Es wird keine zusammengesetzte MP4 gerendert.
- Abbruch beendet den Kindprozess und räumt temporäre Dateien auf. Erneuter
  Import derselben unveränderten Datei stellt Ergebnisse wieder her.
  **KI-Analyse fortsetzen** nutzt abgeschlossene Fenster und Bewertungen weiter.

### Modell, Ressourcen und Grenzen

llama.cpp wird statisch mit eingebettetem Metal in der App gebündelt. Verwendet
wird **Qwen3 1.7B GGUF Q4_K_M**, 1.282.439.264 Bytes, Apache 2.0,
Quantisierung vom llama.cpp-Team `ggml-org`. Downloadquelle und SHA-256 sind in
`crates/media-core/src/models.rs` fest an einen Repository-Commit gebunden;
Dateigröße und vollständiger SHA-256 werden vor Verwendung geprüft.
Originalmodell: <https://huggingface.co/Qwen/Qwen3-1.7B>.
Quantisierung: <https://huggingface.co/ggml-org/Qwen3-1.7B-GGUF/tree/daeb8e2d528a760970442092f6bf1e55c3b659eb>.

Ein gemeinsamer Backend-Lock verhindert parallele Whisper-, Qwen- und
Exportprozesse. Kontext 4096 Tokens, vier Threads, kleine Batches, mmap,
keine vollständigen Video- oder Audiokopien im RAM. Audio wird abschnittsweise
(maximal fünf Minuten) als mono 16-kHz-PCM analysiert. RMS, Spitzen, Änderungen
und Stille sind Zusatzsignale; kein verlässlicher Lach-/Sprecherwechseldetektor.
Jede Inferenz beendet ihren Prozess und gibt Modellressourcen frei; bei
Metal-Problemen einmaliger CPU-Fallback. Auf dem M1 kann dies lange dauern.

Alle Transkriptfenster werden verarbeitet. Für begrenzten Aufwand bleiben maximal
256 zeitlich verteilte Ereignisse, 32 Anker und 96 Bewertungskandidaten erhalten.
Alle gültigen, deduplizierten Bewertungskandidaten sind in der Liste einblendbar.
Globale Nachbarn werden mit semantischen Modellbeschreibungen vorgefiltert;
dadurch können Beziehungen übersehen werden. Dies ist keine Garantie, jeden
besten Moment oder jede irreführende Kombination zu erkennen. Scores (1–10)
sind KI-Einschätzungen, keine Viralitätswahrscheinlichkeit: Unterhaltung 25 %,
Überraschung/Emotion 20 %, Hook 20 %, Klarheit 15 %, Spannung 10 %, Abschluss
10 %. Smart Story wird zusätzlich durch die schwächste Verbindung begrenzt und
leicht pro zusätzlichem Schnitt abgewertet. Mehr Segmente bringen keinen Bonus.

Schnittgrenzen stammen ausschließlich aus Whisper-Segment-/Wortzeiten,
möglichst an Satzenden. Diese Zeiten bleiben ungeprüfte Näherungen.
Keine erfundenen Sekunden und keine verdeckte Verlängerung zu kurzer Szenen.
Daten liegen in `~/Library/Application Support/com.aiclipper.desktop/`
(`models`, `projects`, `analyses`). Kein Upload, keine Telemetrie.

Phase 5 folgt später: mehrteiliger MP4-Export, automatische Endauswahl und
Stapelverarbeitung sowie Untertitel im exportierten Video. Der bisherige
manuelle 9:16-Testexport und die Transkription bleiben verfügbar.

## Phase 3: Englische Transkription

1. Installiere die neue DMG wie unten beschrieben. Keine Entwicklungswerkzeuge oder Terminal-Befehle auf deinem Mac erforderlich.
2. Im Bereich **KI-Analyse · Englische Transkription** prüft die App automatisch, ob sie die eingebettete Whisper-Komponente starten kann.
3. Klicke einmal auf **base.en kostenlos herunterladen**. Das englische Modell benötigt etwa **148 MB** Speicherplatz. Downloadfortschritt und anschließende Integritätsprüfung werden angezeigt. Die Installation erfolgt nur, wenn Dateigröße und **SHA-256** zum festgeschriebenen Modell passen. Ein abgebrochener/fehlerhafter Download wird gelöscht und kann erneut gestartet werden.
4. Importiere ein Video mit englischer Tonspur und klicke **Transkription starten**. FFmpeg extrahiert die erste Tonspur als **Mono-WAV, 16 kHz, PCM 16 Bit**. whisper.cpp transkribiert lokal; es wird keine OpenAI-API und kein Online-Sprachdienst verwendet.
5. Im Transkript kannst du suchen und auf einen Segment- oder Wortzeitstempel klicken, um den Player dorthin zu bewegen. **Transkription / Download abbrechen** beendet den Vorgang. Ein bereits fertiges Transkript bleibt bei einem abgebrochenen Neulauf erhalten.
6. Das Transkript wird automatisch lokal gespeichert. Wählst du nach einem Neustart **dieselbe unveränderte Videodatei am selben Ort** erneut aus, wird das zugehörige Transkript wiederhergestellt. Es muss dafür nicht erneut transkribiert werden.

Nach dem einmaligen Modell-Download funktionieren Import, Transkription und Export offline. Für den Download wird ausschließlich die öffentliche Modelldatei von Hugging Face abgerufen; Videos, Audiodaten und Transkripte werden nicht übertragen. Ein Internetzugang ist nur für den Modell-Download nötig.

**Modell:** `base.en`, Standard für 8 GB RAM. Die getrennte Modellverwaltung und das Transkriptformat sind für spätere Modelle vorbereitet; `small.en` wird in dieser Phase noch nicht angeboten. Es wird englische Sprache erkannt, keine Übersetzung anderer Sprachen erstellt.

**Zeitstempel:** Segmentzeiten und besonders Wortzeiten sind automatisch geschätzt. Die Anzeige „Ungefähre Wortzeiten“ kennzeichnet das ausdrücklich. Das JSON enthält `wordsApproximate`, je Wort `approximate`, `alignmentStatus: unreviewed`, Schema-, Modell- und Engine-Version für spätere Synchronisierung/Korrektur. Dies ist noch kein synchronisierter Untertitel-Export. Whisper kann Wörter auslassen oder halluzinieren, besonders bei Musik, Stille, Überlagerungen und Abschnittsgrenzen; prüfe wichtige Stellen am Original.

### Projekte, Dateien und Speicher

Modelle und Transkripte liegen außerhalb der App unter `~/Library/Application Support/com.aiclipper.desktop/`:

- `models/ggml-base.en.bin`: verifiziertes Modell, bleibt beim App-Update erhalten.
- `projects/<Quellkennung>.json`: vollständiger englischer Text mit Segment- und Wortzeitstempeln.
- `temporary/`: nur die jeweils benötigten Audio-/JSON-Zwischendateien; automatische Bereinigung bei Erfolg, Fehler und normalem Abbruch.

Die Quellkennung berücksichtigt den kanonischen Pfad, Größe, Änderungszeit sowie die ersten/letzten 64 KiB der Datei. Sie vermeidet einen vollständigen Scan großer Videos. Verschieben oder Ändern eines Videos erzeugt eine neue Zuordnung; vollständige Projektverwaltung und Umzugserkennung sind noch nicht enthalten. Eine beschädigte Projektdatei wird verständlich gemeldet und kann durch erneute Transkription ersetzt werden.

Es läuft **höchstens ein speicherintensiver Vorgang gleichzeitig** (Download, Export oder Transkription). Der Player wird beim Start pausiert. Lange Audiospuren werden in gleichmäßig aufgeteilten Abschnitten von höchstens **300 Sekunden** verarbeitet, jeweils als etwa 9,6 MB WAV. Es gibt keine vollständige Videokopie im RAM. Jeder Whisper-Prozess verwendet vier CPU-Threads, einen Prozessor und Greedy-Decoding; er endet nach seinem Abschnitt und gibt Modell-/GPU-Ressourcen frei. Metal ist auf macOS aktiviert und dessen Shader sind im Programm eingebettet; zusätzliche Metal-Dateien oder Xcode auf deinem Mac sind nicht nötig. Schlägt der GPU-Lauf fehl, wird derselbe Abschnitt einmal im CPU-Modus versucht.

Transkripte werden atomar erst nach vollständigem Erfolg gespeichert. Bei Fehler/Abbruch bleiben vorherige vollständige Ergebnisse erhalten. Nach erzwungenem Beenden/Stromausfall können temporäre Dateien übrig bleiben. Die aktuelle Schutzgrenze liegt bei sieben Tagen Quelldauer bzw. 64 MB Projekt-JSON. Die Oberfläche zeigt anfangs höchstens 100 Segmente; weitere lassen sich einblenden. Ein tatsächlicher M1-RAM-/Geschwindigkeitstest für Phase 3 bleibt erforderlich.

### Was folgt später?

Phase 4 ergänzt lokale Szenenerkennung und Ranking (siehe oben). Automatische Endauswahl und Fertigstellung von zehn Clips folgen später. Der manuelle Testclip-Export aus Phase 2 bleibt vollständig nutzbar.

## Was funktioniert außerdem?

- Lokaler Videoimport und Player mit Dauer und Auflösung.
- Ein Testclip mit frei wählbarem Start und **61–90 Sekunden** Länge.
- Automatischer **mittiger 9:16-Bildausschnitt**, skaliert auf **1080 × 1920 Pixel**, 30 fps.
- MP4 mit **H.264 und AAC**, Originalton bleibt erhalten; eine stumme Quelle bleibt stumm.
- Fortschrittsanzeige, Abbruch und automatische Prüfung vor der Erfolgsmeldung.

**Später:** automatische Endauswahl von zehn Szenenkonzepten, intelligentes Reframing, englische Untertitel, Stapel-Export und Link-Import. Der Button „10 Clips erstellen“ bleibt deaktiviert.

## Installieren – ohne Terminal

1. Öffne [GitHub Actions](https://github.com/bestclipper/AI-Clipper/actions) und den neuesten erfolgreichen Lauf von **Validate and build macOS ARM64** (der Phase-4-Pull-Request hat eigene Läufe).
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

Die Dateiendung garantiert keinen kompatiblen Codec. Der FFmpeg-Build enthält native Decoder für verbreitete H.264-/HEVC-/VP8-/VP9-Videos und gängige Audioformate, aber keine externen Codec-Bibliotheken. System-WebView und FFmpeg können unterschiedliche Formate unterstützen. Nicht unterstützte oder beschädigte Quellen erzeugen eine Fehlermeldung. DRM-geschützte Inhalte werden nicht unterstützt. Der MP4-Export enthält noch keine eingebrannten Untertitel; das Transkript aus Phase 3 wird getrennt gespeichert.

## whisper.cpp im installierten App-Paket

whisper.cpp **v1.8.7**, Commit `48f628a84833905ee4a0658ee6d4a5c915ce1997`, wird aus geprüftem Quellcode statisch gebaut. `whisper-cli` liegt als signiertes ARM64-Sidecar direkt neben dem App-Programm. Keine Homebrew-, Python-, OpenAI- oder separate Whisper-Installation erforderlich. Das Modell wird grafisch in den App-Datenordner geladen, nicht bei jedem Start neu heruntergeladen.

Ein zusätzlicher kostenloser Linux-CI-Job lädt das **echte base.en-Modell mit dem Downloader der App** und erkennt eine englische JFK-Sprachaufnahme. Er prüft Text, Wort-/Segmentzeiten, abschnittsweise Verarbeitung, Speichern/Laden und Abbruch. Auf macOS startet das tatsächlich kompilierte App-Programm seine Whisper-Komponente aus einer isolierten `.app`-Verzeichnisstruktur mit Leerzeichen und ohne Repository als Arbeitsverzeichnis. Der ARM64-Bundlecheck prüft zusätzlich die eingebetteten Werkzeuge und Signaturen. Die Intel-Runner können den ARM64-Metal-Lauf selbst nicht ausführen; dieser bleibt Teil des M1-Abnahmetests.

## Kostenloser macOS-ARM64-Build

[Workflow](.github/workflows/build.yml): Frontend-Prüfungen und echte FFmpeg-Integrationstests laufen auf Ubuntu. Der kostenlose Standard-Runner **`macos-15-intel`** baut aus geprüftem, festgeschriebenem FFmpeg-Quellcode die Intel- und ARM64-Werkzeuge. Auf Intel werden die Werkzeuge gestartet und die Filter mit einem Testvideo geprüft; Rust-Tests laufen auf der Hostarchitektur. Tauri erzeugt anschließend die ARM64-App mit `--target aarch64-apple-darwin`.

Vor dem DMG-Upload werden App, FFmpeg und FFprobe auf ARM64 und gültige Ad-hoc-Signaturen geprüft; die DMG wird mit `hdiutil verify` kontrolliert. Die App enthält außerdem FFmpeg-Quellcode und Lizenzhinweise. Ein ARM64-Laufzeittest ist auf dem Intel-Runner nicht möglich.

Das Repository ist öffentlich. [GitHub listet `macos-15-intel` als kostenlosen Standard-Runner](https://docs.github.com/en/actions/reference/runners/github-hosted-runners). Es werden keine kostenpflichtigen größeren Runner oder APIs eingesetzt. [Tauri dokumentiert den ARM64-Cross-Build auf Intel](https://tauri.app/distribute/app-store/). Kostenlose Runner können Warteschlangen haben. Bei privaten Repositories gelten Kontingente; ein Ausgabenlimit von 0 verhindert kostenpflichtigen Mehrverbrauch.

## FFmpeg und Lizenzen

Die Anwendung und whisper.cpp stehen unter MIT; die Whisper-Modellgewichte ebenfalls. FFmpeg n8.0.3 wird als eigenständiges Werkzeug unter LGPL 2.1 oder später mitgeliefert. GPL-/Nonfree-Komponenten und externe Codec-Bibliotheken sind deaktiviert. Quellcode, Lizenztext und exakte Build-Anleitung liegen in der installierten App unter `Contents/Resources/third-party/`. Details: [THIRD_PARTY.md](THIRD_PARTY.md). Es entstehen keine API- oder Lizenzabonnementkosten.

## Entwicklung nur in Codex Cloud

Die folgenden Befehle gehören in die Cloud-Entwicklungsumgebung, **nicht auf deinen Mac**:

```sh
npm ci
npm test
npm run build
npm run test:e2e
cargo test --manifest-path crates/media-core/Cargo.toml --locked
AI_CLIPPER_TEST_FFMPEG=/usr/bin/ffmpeg AI_CLIPPER_TEST_FFPROBE=/usr/bin/ffprobe \
  cargo test --manifest-path crates/media-core/Cargo.toml --locked -- --include-ignored --skip real_english_speech_timestamps_restore_chunking_and_cancel
```

Die echten FFmpeg-Integrationstests sind explizit markiert und werden in CI mit `--include-ignored` verpflichtend ausgeführt. Sie verwenden den Linux-Softwareencoder `libx264` ausschließlich zum Testen; dieser ist nicht in der ausgelieferten App enthalten. Die UI-Vertragstests simulieren die Tauri-Brücke und ersetzen keine native Prüfung. Der separate Speech-CI-Job lädt das echte Modell und führt den ausgelassenen ASR-Test verpflichtend aus. In dieser Codex-Cloud-Umgebung ist der Hugging-Face-Download per Netzwerkregel gesperrt; echte ASR-Läufe werden daher auf GitHub geprüft.

`npm run dev` bietet eine Browser-Vorschau mit Dateiimport. Ein Export ist dort deaktiviert und als Mac-App-Funktion gekennzeichnet. Für native Entwicklung müssen die jeweiligen Sidecars zuerst in der passenden Cloud-Umgebung gebaut werden; `scripts/build-ffmpeg-macos.sh` übernimmt das auf macOS. Linux kann kein vollständiges macOS-Paket erzeugen.

## Architektur und Prüfbericht

`crates/media-core` enthält die unabhängig von Tauri testbare Prozesssteuerung, den Schnitt und die Exportprüfung. `src-tauri` prüft Dateiberechtigungen, öffnet den Exportordnerdialog und begrenzt gleichzeitige Exporte. `src/ExportPanel.tsx` bietet Auswahl, Fortschritt und Ergebnis. Die spätere automatische Fünf-Clip-Pipeline bleibt in `src/pipeline.ts` vorbereitet.

Tatsächlich ausgeführte Prüfungen und die M1-Abnahmeliste stehen in [VALIDATION.md](VALIDATION.md).
