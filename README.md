# MarkIt

## Description

The MarkIt tool helps sound editors to automate detecting scene changes in videos and importing scene markers in Pro Tools.
The main features of the tool include:
- Detecting scene changes.
- Generation of MIDI files with markers at scene change points.
- Easy import of generated MIDI files into Pro Tools sessions.
- User-friendly GUI for configuring detection parameters and exporting results.

**Notes:**
- The MIDI file can be imported into a Pro Tools session that does not contain any markers.
- Pro Tools has a limit of importing maximum 1.000 markers in a session.

## Motivation

The motivation for this project was to help a friend automate a tedious and boring part of the sound editing process.

## Dev notes

This section contains useful information for developers who are interested in the implementation of MarkIt.

### Installing requirements

Requirements are listed in the `requirements.txt` file.
They can be installed with the following command:

```
pip install -r requirements.txt
```

### Running the tool

Main file is `gui/gui.py`.
To start the tool you should run this file.

### Creating a portable executable

Portable executable is created using `pyinstaller` tool.

This command can be used to create a portable `.exe` of MarkIt:

```
pyinstaller --noconsole --name MarkIt --icon=markit_icon.ico --onefile gui.py
```

The previous command will create a MarkIt.exe file in `dist` folder.
It will also create a `MarkIt.spec` file.

The `.spec` file can be customized and used to create different version using the following command:

```
pyinstaller MarkIt.spec
```