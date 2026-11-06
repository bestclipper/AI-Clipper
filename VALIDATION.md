# Prüfbericht · Phase 2 (v0.2.0)

Entwicklung und Befehle ausschließlich in Codex Cloud. Phase 1 wurde vom Nutzer auf dem MacBook Air M1 mit 8 GB erfolgreich getestet: App-Start, Import und Vorschau.

## In Codex Cloud ausgeführt

| Prüfung | Ergebnis |
| --- | --- |
| TypeScript und Vite-Produktionsbuild | erfolgreich |
| Frontend-Unit-Tests | 7 erfolgreich |
| Browser-Integrationstests | 5 erfolgreich mit installiertem Chromium |
| Rust-Medienkern-Unit-Tests | 2 erfolgreich |
| Echte FFmpeg-Integrationstests | 2 erfolgreich, explizit mit `--include-ignored` |
| Rust-Formatierung, Prettier, Shellsyntax und Git-Diff-Prüfung | erfolgreich |
| Vollständige native Tauri-App / ARM64-DMG | Prüfung durch neuen macOS-CI-Lauf; Ergebnis im Pull Request |

Die Browsertests decken weiterhin Import, Wiedergabe, Dateiwechsel und Fehler ab. Neue UI-Vertragstests simulieren die Tauri-Brücke für Fortschritt, Sperren der Eingaben, Fehler, Ordnerdialog-Abbruch und Export-Abbruch. **Sie führen selbst kein FFmpeg aus**.

Die getrennten Rust-Integrationstests führen echtes FFmpeg/FFprobe unter Linux aus: synthetische 63-Sekunden-Quellen mit und ohne AAC-Ton, Landschafts- und Hochkantformat, Zuschnitt auf 1080 × 1920, 61-Sekunden-H.264-Export, Metadatenprüfung und vollständige Dekodierung. Zusätzlich werden Fortschritt, Abbruch eines laufenden Prozesses, Bereinigung, beschädigte Dateien, ungültige Zeitbereiche, Dateinamen mit Leerzeichen und Schutz vorhandener Dateien geprüft. Linux verwendet den Softwareencoder libx264 zum Testen. Die ausgelieferte App verwendet VideoToolbox und enthält keinen libx264-Encoder.

Befehle in dieser Cloud:

```sh
npm test
npm run build
PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/usr/bin/chromium npm run test:e2e
AI_CLIPPER_TEST_FFMPEG=/usr/bin/ffmpeg AI_CLIPPER_TEST_FFPROBE=/usr/bin/ffprobe \
  cargo test --manifest-path crates/media-core/Cargo.toml --locked -- --include-ignored
```

## Verpflichtende CI-Prüfungen

- Ubuntu: Frontend-Tests, Browsertests und alle echten FFmpeg-Integrationstests.
- macos-15-intel: FFmpeg n8.0.3 aus festgeschriebenem Commit mit verifiziertem SHA-256 bauen, Intel-/ARM64-Architekturen und Systembibliotheksabhängigkeiten prüfen.
- Mitgelieferte Intel-Werkzeuge starten, FFprobe und den tatsächlichen Crop-/Scale-/Audiofilter mit der Testdatei prüfen, ohne einen VM-Hardwareencoder vorauszusetzen.
- Native Tauri-Kompilierung und Rust-Prüfungen auf Intel; Cross-Build für aarch64-apple-darwin.
- App, FFmpeg und FFprobe müssen reine ARM64-Binaries und ad-hoc signiert sein. Die vollständigen FFmpeg-Quellen und Lizenzdateien müssen in der App liegen. DMG-Prüfsumme muss gültig sein.

Die auf dem Intel-Runner gebauten ARM64-Werkzeuge können dort nicht ausgeführt werden. Ein erfolgreicher Build ersetzt keinen Funktionstest des VideoToolbox-Encoders, des nativen Dateidialogs und der installierten App auf einem echten M1.

## Abnahme auf dem M1

1. Phase-2-DMG installieren und App per Finder starten. Ohne separate FFmpeg-Installation testen.
2. Lokalen Stream mit mindestens 90 Sekunden und Ton importieren. Einen Abschnitt ab einer Stelle ungleich 0 wählen und 61 Sekunden exportieren. Danach 90 Sekunden testen.
3. MP4 in QuickTime abspielen: gewünschter Anfang, Originalton, 9:16-Beschnitt, Bild/Ton-Synchronität und vollständiges Ende prüfen.
4. Stumme, bereits vertikale und gedrehte Quellen prüfen. Der mittige Beschnitt ist beabsichtigt; es gibt noch kein intelligentes Reframing.
5. Fortschrittsanzeige und Verifikationsphase prüfen. Während des Exports dürfen Quelle und Zeitauswahl nicht gewechselt werden.
6. Export abbrechen und App danach erneut verwenden. Exportordnerdialog abbrechen. Nicht beschreibbaren/vollen Datenträger prüfen.
7. Während eines Exports schließen/beenden: zunächst muss abgebrochen und bereinigt werden; anschließend App erneut schließen.
8. Ohne Internet erneut exportieren. Originaldatei und vorhandene Exporte müssen unverändert bleiben.
9. Langen hochauflösenden Stream unter Beobachtung des Arbeitsspeichers auf dem M1 mit 8 GB prüfen. Es wird nur ein Exportprozess gestartet; ein empirischer RAM-Höchstwert ist noch nicht nachgewiesen.

Bei erzwungenem Prozessabbruch/Stromausfall kann eine unvollständige Ausgabe liegen bleiben. Ein solches Ergebnis hat keine erfolgreiche Abschlussprüfung erhalten.
