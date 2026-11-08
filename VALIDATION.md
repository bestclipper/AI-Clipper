# Prüfbericht · Phase 3 / v0.3.0

Stand: 8. November 2026. Alle Entwicklungsbefehle in Codex Cloud und GitHub Actions, keine Befehle auf dem Mac des Nutzers. Phase 1 und 2 wurden vom Nutzer auf einem MacBook Air M1 mit 8 GB erfolgreich abgenommen, inklusive 9:16-MP4-Export.

## In Codex Cloud ausgeführt

- TypeScript und Vite-Produktionsbuild: erfolgreich.
- Frontend-Unit-Tests: 9 erfolgreich.
- Browser-Integrationstests: 9 erfolgreich (Import, Wiedergabe, Export-UI, Download-UI, Abbruch, Transkriptanzeige, Zeitsprung und Wiederherstellung).
- Rust-Kerntests: SHA-256/Größenprüfung, fehlende/beschädigte Modelle, Zeitstempel, lange Audioabschnittspläne, atomare Speicherung/Wiederherstellung und Beenden eines laufenden Prozesses.
- Echte FFmpeg-Tests: bisheriger H.264/AAC-Export sowie Mono-16-kHz-PCM-Extraktion aus einer über 300 Sekunden langen Videoquelle; beschädigte Eingaben und Bereinigung geprüft.
- whisper.cpp v1.8.7 aus festgeschriebenem Quellcode unter Linux erfolgreich gebaut und `whisper-cli --version` gestartet.
- Rust-Formatierung, Prettier, Shellsyntax und Git-Diff-Prüfung.

Browser-Vertragstests simulieren ausschließlich die Tauri-Brücke. Die echten Medien-/Spracherkennungstests sind davon getrennt. Ein Hugging-Face-Abruf aus dieser Cloud wurde mit HTTP 403 durch die Umgebungsrichtlinie abgelehnt. Es wurde keine Umgehung eingerichtet. Der tatsächliche Modell-Download und die englische Erkennung laufen deshalb im separaten GitHub-Speech-Job.

## Verpflichtende GitHub-Prüfungen

1. Frontend: alle bisherigen und neuen Unit-/Browsertests.
2. Medienkern: Rust-Tests und echte FFmpeg-Exporte/Audioextraktion.
3. Speech: echte base.en-Datei mit dem **Downloader der Anwendung** herunterladen, Bytezahl und SHA-256 prüfen; JFK-Sprachaufnahme zu einem Video verarbeiten; tatsächliches whisper.cpp ausführen. Erwartete englische Wörter („ask not“, „country“), Zeitstempel und vorhandene Wortzeiten prüfen. Dasselbe Video in mehreren kleinen Abschnitten transkribieren, globale Zeitstempel und Speichern/Laden prüfen. Neulauf abbrechen und Erhalt des alten Transkripts sowie entfernte Temporärdateien bestätigen. Fehlendes Modell muss eine Fehlermeldung ergeben.
4. macos-15-intel: FFmpeg/FFprobe einschließlich WAV-Muxer und whisper.cpp mit statisch eingebettetem Metal-Backend für Intel und ARM64 bauen. Keine fremden Homebrew-/OpenMP-Dylibs zulassen.
5. Mitgeliefertes FFmpeg muss tatsächlich Mono-16-kHz-PCM-WAV erzeugen können.
6. Das native App-Programm wird in einer isolierten `.app/Contents/MacOS`-Struktur aus einem fremden Arbeitsverzeichnis mit `--check-whisper` ausgeführt und muss sein Whisper-Sidecar erfolgreich starten. Diese Diagnose verwendet dieselbe Pfadauflösung wie die eigentliche Transkription und die UI-Komponentenprüfung.
7. ARM64-App erstellen, Architektur und Ad-hoc-Signaturen für App/FFmpeg/FFprobe/Whisper prüfen; Quell-/Lizenzressourcen und DMG-Integrität prüfen; neues DMG als Artefakt hochladen.

Das tatsächliche Ergebnis des nativen Builds und der Downloadlink werden im Phase-3-Pull-Request dokumentiert. Der Intel-Runner kann die ARM64-App nicht ausführen und bietet keinen M1-Metal-Laufzeittest.

## M1-Abnahme der installierten DMG

- Neue App im Finder installieren. Die UI muss „whisper.cpp wurde von der App erfolgreich gestartet“ anzeigen.
- base.en grafisch herunterladen: Fortschritt, Abbruch, erneuter Versuch und erfolgreiche Integritätsprüfung testen.
- Englischsprachiges Video importieren und transkribieren. Prüfen, ob Text und Anfang/Ende der Segmente plausibel sind. Wortzeiten sind ausdrücklich nur ungefähr.
- Auf Segment-/Wortzeiten klicken und zur richtigen Stelle im Video springen. Vorhandenen MP4-Export erneut testen.
- App schließen, erneut starten und dasselbe unveränderte Video importieren: gespeichertes Transkript muss wieder erscheinen, auch ohne Internet.
- Langen Stream transkribieren, Fortschritt über mehrere Abschnitte und Speicherverbrauch in der Aktivitätsanzeige prüfen. Es darf nur ein Modellprozess gleichzeitig laufen. Nach Abschluss muss er beendet sein.
- Transkription abbrechen; erneuter Start muss möglich sein. Vorherige vollständige Transkripte bleiben erhalten. Fehler bei stummer/beschädigter Quelle und vollem Datenträger prüfen.
- Metal-Verhalten und CPU-Fallback auf dem tatsächlichen M1 prüfen. Ein empirischer Phase-3-RAM-/Geschwindigkeitswert wird ohne diesen Gerätetest nicht behauptet.

Erzwungenes Beenden/Stromausfall kann temporäre Dateien hinterlassen. Die Quellzuordnung ist ein effizienter Fingerabdruck aus Pfad, Dateimetadaten und Dateirändern; sie ist kein vollständiger Video-Hash und keine automatische Umzugserkennung.

## Phase 4 (work in progress)

Local checks and CI results will be updated before completion. The real Qwen
model cannot be downloaded in this restricted Codex Cloud environment; the
free GitHub Actions `scene-tests` job performs the actual download, integrity
check and inference. Frontend native IPC mocks test UI contracts, not inference.
The Intel macOS runner executes the actual application binary in a standalone
`.app/Contents/MacOS` layout to start its bundled llama.cpp component; ARM64
architecture, signatures, resources and DMG integrity are checked separately.
Actual M1 GUI inference, Metal utilization and peak memory cannot be verified
on an Intel runner and require an installed-app check on the M1.
