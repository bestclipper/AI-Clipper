# Prüfbericht · Phase 4 Fehlerbehebung / v0.4.1

Stand: 10. November 2026. Alle Entwicklungsbefehle in Codex Cloud und GitHub
Actions. Keine Terminal-Aktionen auf dem Mac des Nutzers.

## Ursache und Reproduktion

Die bisherige Analyse beendet bei einem JSON-Parsefehler den gesamten Lauf,
bevor `evaluated` aktualisiert wird. Beim Fortsetzen wird deshalb derselbe
Vergleich mit demselben Seed und denselben 900 Ausgabetokens wiederholt. Freie
Textfelder hatten keine Längenbegrenzung; Thinking war nur über Prompt und Präfix
abgeschaltet. Der Fehler ist durch einen gespeicherten Stand 28/28 Fenster,
1/20 Vergleiche und dauerhaft abgeschnittene Antworten automatisiert reproduziert.
Nach dem Fix werden alle verbleibenden Vergleiche verarbeitet, vorhandene
Kandidaten bewahrt, höchstens zwei Versuche verbraucht und fünf unterschiedliche
Szenen über gekennzeichnete Ersatzbewertungen ausgewählt. Ein erneuter Aufruf
wiederholt abgeschlossene Fehlstellen nicht.

## Automatisierte Prüfungen

- Rust: 38 Kerntests einschließlich unvollständigem/abgeschnittenem JSON,
  ungültigem UTF-8, Zusatzdaten, Ausgaben über 24 KiB, atomarem Speichern/Laden,
  Migration alter Checkpoints, 28/28–1/20-Fortsetzung, dauerhaften Einzelproblemen,
  wiederhergestellten transienten Fehlern und Abbruch ohne verbrauchten Versuch.
- Fünfer-Auswahl mit sechs geeigneten zusammenhängenden Szenen, korrekten Dauern,
  begrenzter Überlappung und einem Gegenbeispiel zur gierigen Auswahl: ein hoch
  bewerteter überlappender Clip darf fünf bessere unterschiedliche Möglichkeiten
  nicht auf vier reduzieren. Fast identische Schnittgrenzen bleiben bis zur
  Auswahl erhalten, wenn nur eine davon fünf kompatible Szenen ermöglicht. Routine-Sprache erzwingt keine Füllclips.
- Frontend: 14 Unit-Tests und 12 Browser-Vertragstests. Teilfehler dürfen nicht als
  erfolgreiche Vollanalyse erscheinen; Ersatzbewertung, echte Auswahlanzahl,
  Fortschritt, Abbruch, Wiederherstellung und vorhandene Import-/Exportfunktionen.
- Bestehende echte FFmpeg-Tests einschließlich >300 Sekunden Audioextraktion,
  9:16-/H.264-/AAC-Export und Bereinigung bleiben Bestandteil des Workflows.
- Der tatsächliche gepinnte llama.cpp-Build akzeptiert `--reasoning-budget 0`
  und `--reasoning-format none`; dies wurde ohne Modelldatei am CLI geprüft.

## Echte Modelltests und macOS-Build

Hugging Face ist in dieser Codex-Cloud-Umgebung gesperrt. Die echten Modelltests
laufen daher auf kostenlosen GitHub-Runnern, ohne die Netzrichtlinie zu umgehen.
Der Speech-Job führt Whisper base.en mit der echten englischen JFK-Aufnahme aus.
Der Scene-Job prüft mit tatsächlichem Qwen weiterhin den 77-Sekunden-Fall,
Ablehnung unverbundener Szenen, Single Scene und das echte Whisper-Transkript.
Zusätzlich bewertet Qwen fünf unterschiedliche vollständige humorvolle englische
Testgeschichten mit jeweils 75 Sekunden. Die Fünfer-Auswahl muss alle fünf wählen.
Diese Geschichten sind synthetische Transkript-Fixtures; es wird nicht behauptet,
dass sie echte Streamer-Aufnahmen sind.

Der bekannte kostenlose `macos-15-intel`-Workflow erzeugt die ARM64-App und DMG.
Er prüft Komponentenausführung durch das App-Programm, Paketpfade, statische
Komponenten mit eingebettetem Metal, ARM64-Architekturen, Signaturen und die
Integrität der gemounteten DMG. Eine separate Paketkopie mit Intel-Testprogrammen
prüft die Startpfade; die ausgelieferte ARM64-App wird nicht verändert.

Die endgültigen CI-Ergebnisse und der DMG-Link werden im neuen Pull Request
festgehalten. Ein Intel-Runner kann die ARM64-App nicht ausführen. Tatsächliche
M1-GUI-/Metal-Auslastung und Spitzen-RAM bleiben deshalb ein Gerätetest.

## Prüfung auf dem M1 ohne Terminal

1. v0.4.1 aus der neuen DMG installieren und die alte App vorher beenden.
2. Dasselbe unveränderte Video öffnen. Modelle und Transkript müssen vorhanden
   bleiben; abgeschlossene Fenster werden nicht erneut transkribiert oder erkannt.
3. Analyse fortsetzen: Die bisher problematische Stelle muss nach maximal zwei
   Versuchen passiert werden. Fortschritt muss über 1/20 hinausgehen.
4. Nach Ende die fünf ausgewählten Szenen ansehen, Ersatzbewertungen und eventuelle
   Teilfehler prüfen. Bei weniger Treffern müssen Anzahl und Begründung erscheinen.
5. App neu starten: Ergebnisse und Auswahl müssen wiederhergestellt werden.
6. Bestehende Transkription, manuelle MP4-Ausgabe und Playlist-Vorschau prüfen.
7. Bei einem langen Stream RAM und Metal in der Aktivitätsanzeige beobachten.

Keine neuen Rendering-Funktionen: mehrteilige MP4-Ausgabe und Stapel-Export
bleiben außerhalb dieser Fehlerbehebung. Wort-/Segmentzeiten sind weiterhin
ungeprüfte Whisper-Näherungen; Scores sind keine Viralitätsgarantien.
