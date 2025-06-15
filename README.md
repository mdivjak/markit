# MarkIt

![Python](https://img.shields.io/badge/python-v3.7+-blue.svg)
![License](https://img.shields.io/badge/license-GPL%20v3-blue.svg)
![Platform](https://img.shields.io/badge/platform-windows-lightgrey.svg)
![Release](https://img.shields.io/github/v/release/mdivjak/markit?include_prereleases)
![Downloads](https://img.shields.io/github/downloads/mdivjak/markit/total)

## Description

The MarkIt tool helps sound editors to automate detecting scene changes in videos and importing scene markers in Pro Tools.
The main features of the tool include:
- Detecting scene changes.
- Generation of MIDI files with markers at scene change points.
- Easy import of generated MIDI files into Pro Tools sessions.
- User-friendly GUI for configuring detection parameters and exporting results.

## Quickstart

### Download MarkIt

1. Go to the [Releases page](../../releases)
2. Download the latest `MarkIt.exe` file
3. Double-click the downloaded file to run MarkIt

### How to Use

1. **Launch MarkIt** - Double-click the `MarkIt.exe` file
2. **Load your video** - Select your video file using the file browser
3. **Set the output destination** - The output destination for the MIDI file
4. **Detect scenes** - Click the detection button to analyze your video
5. **Import to Pro Tools** - Load the MIDI file into your Pro Tools session

**Important Notes:**
- Only import MIDI files into Pro Tools sessions that don't already have markers
- Pro Tools supports a maximum of 1,000 markers per session

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