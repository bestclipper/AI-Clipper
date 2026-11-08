# Prüfbericht · Phase 4 / v0.4.0

Stand: 8. November 2026. Entwicklung ausschließlich in Codex Cloud und GitHub
Actions. Kein Terminal und keine Entwicklungsinstallation auf dem Mac des
Nutzers. Phase 1–3 wurden vom Nutzer auf dem M1 mit 8 GB erfolgreich abgenommen.

## Lokal erfolgreich

- 12 Frontend-Unit-Tests, TypeScript- und Vite-Produktionsbuild.
- 25 Rust-Kerntests: gewichtete Bewertung, 77-Sekunden-Smart-Story mit bestehenden
  Grenzen 18–45 und 170–220, 3–5 Segmente, Überlappungen, Reihenfolge, Grenzen,
  Belegzitate, Ablehnung unverbundener Beziehungen, Duplikate, 1000 Textstellen,
  kurze Satzsegmente, zeitliche Abdeckung trotz Kandidatenbudget, Checkpoints,
  Integrität, fehlende Modelle und laufender Prozessabbruch.
- Simulierter Speichermangel im Test-Kindprozess: verständlicher Fehler,
  keine erfundenen Kandidaten, temporäre Promptdateien werden entfernt.
  Dies ist ausdrücklich kein gemessener M1-Speichertest.
- Vier echte FFmpeg-Integrationstests: H.264/AAC/9:16-Export, stumme/hochkant
  Quelle, Abbruch, Audioextraktion aus >300 Sekunden Video, beschädigte Eingabe,
  lokale RMS-/Stilleanalyse und Bereinigung.
- llama.cpp aus festgeschriebenem Quellcode unter Linux gebaut und CLI-Optionen
  am tatsächlich gestarteten Programm geprüft.
- Rustfmt, Prettier, Shellsyntax und Git-Diff-Prüfung.

11 Frontend-Browsertests simulieren die Tauri-Brücke und prüfen Import, Export,
Whisper-UI sowie Qwen-Download, Abbruch, gemeinsame Verarbeitungssperre,
Wiederherstellung, automatische Transkript-Aktualisierung und die Wiedergabe
aller Playlist-Teile inklusive Lückensprung und Stopp am letzten Ende. Die kurze
Browser-Videodatei verwendet kurze Vorschauintervalle; die 61–90-Sekunden-Regel
wird separat im Rust-Kern geprüft.

## Echte KI- und macOS-Prüfung in GitHub Actions

Der Hugging-Face-Download ist in dieser Codex-Cloud-Umgebung gesperrt. Keine
Umgehung eingerichtet. Deshalb lädt der kostenlose GitHub-Runner das tatsächlich
verwendete Qwen-Modell über den App-Downloader und prüft Bytezahl und SHA-256.

`speech-tests` transkribiert die echte englische JFK-Aufnahme mit base.en,
prüft Text, Wort-/Segmentzeiten, mehrere Abschnitte, Speicherung und Abbruch.
Das echte Transkript wird als Testartefakt an `scene-tests` weitergegeben.
`scene-tests` führt tatsächliches Qwen aus: Ereignisse aus getrennten Fenstern,
Smart-Story-Planung, inhaltliche Verbindung von Versprechen und späterem Ergebnis,
77 Sekunden Gesamtdauer, unverbundene Rezept-/Spielszene als Negativkontrolle,
Single-Scene-Bewertung sowie Qwen-Verarbeitung des tatsächlichen Sprachtranskripts.

Der kostenlose Standard-Runner `macos-15-intel` baut statische FFmpeg-, Whisper-
und llama.cpp-Komponenten mit eingebettetem Metal für Intel und ARM64. Das
App-Programm startet die mitgelieferten Komponenten aus einem unabhängigen
Arbeitsverzeichnis. ARM64-Architektur, Ad-hoc-Signaturen, Lizenz-/Quellressourcen
und DMG-Integrität werden geprüft. Die DMG wird gemountet und ihre ARM64-Dateien
werden geprüft. Eine separate Kopie des DMG-Bundles erhält Intel-Testprogramme,
damit die Pfadauflösung der verpackten App auch auf dem Intel-Host ausführbar ist.
Die ausgelieferte ARM64-App wird dabei nicht verändert.

**Grenze:** Ein Intel-Runner kann die ARM64-App nicht ausführen. Ein tatsächlicher
M1-GUI-Lauf, Metal-Auslastung und Spitzen-RAM-Verbrauch sind nicht in der Cloud
verifiziert. Die Komponentenprüfung startet llama.cpp; sie ersetzt keinen
vollständigen Qwen-Lauf in der ARM64-GUI. Der zusätzliche native Linux-Tauri-
Check scheitert in Codex Cloud an fehlenden GTK-/GLib-Systembibliotheken;
macOS-Kompilierung wird im dafür vorgesehenen GitHub-Job geprüft.

Der endgültige CI-Status und der DMG-Link stehen im Pull Request #4.

## M1-Abnahme ohne Terminal

1. Neue DMG im Finder installieren, ältere App vorher beenden und ersetzen.
2. Die UI muss anzeigen, dass llama.cpp von der App erfolgreich gestartet wurde.
3. Qwen grafisch herunterladen, einmal abbrechen und erneut laden; anschließend
   muss die Integritätsprüfung erfolgreich sein. Vorhandenes base.en bleibt erhalten.
4. Englischen Stream importieren, KI-Analyse starten. Bei fehlendem Transkript
   muss zuerst Whisper laufen; das Transkript erscheint danach direkt oben.
5. Single Scene und Smart Story ansehen. Teile müssen in der richtigen
   Reihenfolge spielen; Lücken werden übersprungen. Zeiten sind nur ungefähr.
6. Analyse abbrechen und fortsetzen, App neu öffnen und dieselbe unveränderte
   Quelldatei importieren. Ergebnisse müssen wiederhergestellt werden.
7. Ohne Internet erneut analysieren und bisherigen manuellen MP4-Testexport prüfen.
8. In der Aktivitätsanzeige bei einem langen Stream Speicher und Metal-Verhalten
   prüfen. Whisper und Qwen dürfen nicht gleichzeitig Modelle verarbeiten;
   beendete Kindprozesse dürfen keine Modellressourcen weiter halten.

KI-Qualität bleibt modellbedingt begrenzt. Es werden keine zehn Treffer erzwungen
und keine sichere Erkennung jedes Zusammenhangs oder jeder Irreführung behauptet.
Keine mehrteilige MP4-Ausgabe in Phase 4. Erzwungenes Beenden/Stromausfall kann
Temporärdateien hinterlassen; normaler Abbruch räumt auf.
