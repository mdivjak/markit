# MarkIt

## Motivation

The motivation for this project was to help a friend automate a tedious and boring part of the sound editing process.

## Description

The MarkIt tool helps sound editors to automate detecting scene changes in videos and importing scene markers in Pro Tools.
The main features of the tool include:
- Detecting scene changes.
- Generation of MIDI files with markers at scene change points.
- Easy import of generated MIDI files into Pro Tools sessions.
- User-friendly GUI for configuring detection parameters and exporting results.

This MIDI file can be imported into a Pro Tools session that does not contain any markers.

## Commands

These commands are used to create a portable .exe of MarkIt:

```
pyinstaller --noconsole --name MarkIt --icon=markit_icon.ico --onefile gui.py
pyinstaller MarkIt.spec
```

## Release Notes

- 15.12.2024. fixed tick calculation for markers and updated .exe app
- 07.01.2025. fixed bug in tick calculation in the GUI version of the app
- 14.02.2025. added field for entering fps values
