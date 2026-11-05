# AI Clipper · Phase 1

Kostenlose lokale macOS-Anwendung für Apple Silicon, entwickelt mit **Tauri 2, React und TypeScript**. Zielgerät: MacBook Air M1 mit 8 GB RAM, macOS 12 oder neuer.

## Was funktioniert bereits?

- Dark-Mode-Oberfläche mit lokalem Dateiauswahldialog.
- Import einer MP4-, MOV-, M4V- oder WebM-Datei ohne Upload oder Kopie.
- Video-Vorschau mit Wiedergabe/Pause, Zeitleiste, Lautstärke und Vollbild über die Player-Steuerung.
- Anzeige von Dateiname, Dauer und Videoauflösung.
- Video wechseln oder entfernen; verständliche Fehlermeldungen.
- Vorbereitung einer lokalen Verarbeitungspipeline mit abbrechbarer Analyse-/Export-Schnittstelle.

**Noch nicht enthalten:** KI-Szenenauswahl, Transkription, Untertitel, Hochkant-Konvertierung und MP4-Export. Die Schaltfläche „5 Clips erstellen“ ist deshalb deaktiviert. Das Clip-Profil zeigt die spätere Zielkonfiguration, keine bereits erzeugten Clips. Es gibt keine manuelle Szenenauswahl.

## Auf deinem Mac installieren – ohne Terminal

1. Öffne im GitHub-Repository den Reiter **Actions** und den neuesten erfolgreichen Lauf von **Validate and build macOS ARM64**. Für einen ersten Test kannst du auch den Lauf des Pull Requests verwenden.
2. Lade unten unter **Artifacts** das Archiv **AI-Clipper-macOS-ARM64** herunter (GitHub-Anmeldung erforderlich; Artefakte bleiben 14 Tage verfügbar).
3. Entpacke das ZIP im Finder und öffne die enthaltene `.dmg`-Datei.
4. Ziehe **AI Clipper** nach **Programme** und starte die App dort.
5. Klicke **Video auswählen** und wähle eine lokale Datei. Anschließend kannst du das Video direkt ansehen.

Du brauchst dafür weder Node.js, Rust, Homebrew noch FFmpeg auf deinem Mac zu installieren.

### macOS-Sicherheitshinweis

Der kostenlose Build ist **ad-hoc signiert, aber nicht mit einer Apple Developer ID signiert und nicht notarisiert**. macOS kann den ersten Start blockieren. Öffne nach dem ersten Startversuch **Systemeinstellungen → Datenschutz & Sicherheit → Dennoch öffnen**, falls macOS diese Option anbietet, und bestätige den Start. Öffne nur einen Build aus diesem Repository, dessen Herkunft du geprüft hast. Falls dein macOS oder eine Geräteverwaltung das Öffnen weiterhin verhindert, ist die Installation dort ohne eine andere Verteilungs-/Signierungslösung nicht garantiert. Eine pauschal deaktivierte Gatekeeper-Prüfung ist nicht erforderlich. Eine reibungslose notarisierten Verteilung würde eine kostenpflichtige Apple-Mitgliedschaft erfordern und gehört nicht zu diesem kostenlosen Aufbau.

## Formate und Arbeitsspeicher

Dateiendungen garantieren keine Codec-Unterstützung. Der macOS-Player verwendet WKWebView und die System-Codecs. **MP4 mit H.264 und AAC** ist die empfohlene Kombination. MOV/M4V/WebM hängen von Codec und macOS-Version ab; MKV/AVI werden in Phase 1 nicht angeboten. Bei inkompatiblen oder beschädigten Dateien zeigt die App eine Fehlermeldung. Phase 1 enthält keinen Konverter.

Das Video wird über Tauri Asset Protocol mit bedarfsgerechten Dateizugriffen abgespielt. Es wird nicht als Base64 oder vollständig in JavaScript geladen. Es ist immer nur eine Quelle aktiv, die Vorschau lädt zunächst nur Metadaten. Große Auflösungen oder hohe Bitraten können den M1 trotzdem belasten; ein RAM-/Langzeitbenchmark auf einem echten M1 mit 8 GB steht noch aus. Die importierte Quelle bleibt unverändert. Projekte werden noch nicht gespeichert; nach dem Neustart wählst du die Quelle erneut aus.

## Datenschutz und kostenlose Technologien

Die gebündelte Anwendung benötigt zur Laufzeit keine Internetverbindung, API-Schlüssel oder Konten. Kein Tracking, keine Remote-Schriftarten und keine Uploads. Die Content Security Policy erlaubt lokale Inhalte und Tauri-IPC; der native Dialog gewährt Zugriff auf ausgewählte Dateien, ohne das gesamte Dateisystem pauschal freizugeben. Der Browsermodus für Entwicklung verwendet kurzlebige Blob-URLs und gibt sie beim Dateiwechsel frei.

Tauri, React, TypeScript, Vite, Rust und Lucide sind kostenlose Open-Source-Technologien; der Anwendungscode steht unter MIT. Es werden keine kostenpflichtigen APIs verwendet. GitHub Actions ist für öffentliche Repositories mit Standard-Runnern kostenlos; private Repositories haben ein begrenztes Inklusivkontingent. Stelle bei privaten Repositories das Actions-Ausgabenlimit auf 0, wenn keinerlei Mehrkosten entstehen dürfen. Ohne verfügbares Kontingent kann der Build dann aussetzen. Es wird kein kostenpflichtiger größerer Runner verwendet.

## Entwicklung ausschließlich in Codex Cloud

Die folgenden Befehle sind für die **Cloud-Entwicklungsumgebung**, nicht für deinen Mac:

```sh
npm ci
npm test
npm run build
npx playwright install --with-deps chromium
npm run test:e2e
```

`npm run dev` startet die Browser-Vorschau auf Port 1420. Der native Desktop-Dateidialog und das Asset Protocol sind nur in Tauri aktiv. Die Browser-Vorschau verwendet einen lokalen HTML-Dateidialog.

Für native Entwicklung in einer entsprechend ausgestatteten Cloud-Umgebung: `npm run tauri -- dev`. Unter Linux werden dafür Rust und die Tauri-Systembibliotheken (u. a. WebKitGTK 4.1) benötigt. Ein macOS-DMG lässt sich nicht vollständig unter Linux bauen; der Workflow baut auf einem macOS-Runner mit Ziel `aarch64-apple-darwin`.

## Automatischer ARM64-Build

[Workflow](.github/workflows/build.yml): Bei Pull Requests, Pushes auf `main`/`master`, `v*`-Tags und manueller Ausführung werden zuerst Frontend-Tests, TypeScript/Produktionsbuild und Browser-Integrationstests ausgeführt. Danach prüft der macOS-Job Rust-Formatierung und Rust-Tests und erstellt `.app` und `.dmg` für Apple Silicon. Die DMG wird als Download-Artefakt bereitgestellt; es wird kein Release automatisch veröffentlicht. Unter **Actions → Validate and build macOS ARM64 → Run workflow** lässt sich ein abgelaufenes Artefakt neu erstellen. Erforderlich: Actions und Workflow-Ausführungen müssen im Repository erlaubt sein.

## Grundlage für Phase 2

`src/pipeline.ts` definiert `LocalPipeline`, `ClipCandidate`, `PipelineStage` und das unveränderliche Ausgabeprofil:

| Einstellung | Ziel |
| --- | --- |
| Szenenwahl | automatisch, fünf unterhaltsame Highlights |
| Clip-Länge | 61–90 Sekunden |
| Bildformat | 9:16, 1080 × 1920 |
| Untertitel | Englisch |
| Export | fünf MP4-Dateien |

Später können lokale Worker für z. B. whisper.cpp und FFmpeg angebunden werden. Diese sind in Phase 1 **nicht integriert** und werden nicht heruntergeladen. Modellwahl, Modelllizenzen, englische Transkription bzw. Übersetzung, Rankingqualität und CPU-/RAM-Budget müssen in Phase 2 auf dem M1 geprüft werden. Geplant ist sequenzielle Verarbeitung, damit keine fünf speicherintensiven Jobs gleichzeitig laufen. Die Schnittstelle berücksichtigt einen Abbruch über `AbortSignal`.

## Prüfstatus und Grenzen

Siehe [VALIDATION.md](VALIDATION.md) für die tatsächlich ausgeführten Prüfungen. Browsertests decken Import, Metadaten, echte Wiedergabe, Dateiwechsel, Entfernen und Fehlerfälle ab. Das kleine H.264/AAC-Testvideo wurde synthetisch mit FFmpeg erzeugt und enthält keine fremden Inhalte.

**Wichtig:** Browsertests ersetzen keinen nativen macOS-Test. Der macOS-Dateidialog, dynamische Asset-Freigaben, WKWebView-Codecs, DMG-Installation und Gatekeeper müssen zusätzlich auf macOS geprüft werden. Ein erfolgreicher GitHub-Build belegt die Paketierung, aber keinen vollständigen Funktionstest auf einem MacBook Air M1.
