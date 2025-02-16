# Scene Detector

Ovaj projekat je alat koji detektuje scene u video fajlovima.
Projekat kreira fajl sa markerima koji moze da se importuje u Pro Tools.
Cilj ovog projekat je da pomogne montazerima zvuka da ubrzaju rad na svom projektu.

## Commands

```
pyinstaller --noconsole --name MarkIt --icon=markit_icon.ico --onefile gui.py
pyinstaller MarkIt.spec
```

## Dependencies

```
pip install scenedetect[opencv]
pip install pyaaf2
```

## Release Notes

- 15.12.2024. popravio racunanje ticka za markere i update-ovao .exe app
- 07.01.2025. popravio bug u racunanju tickova u gui verziji app
- 14.02.2025. dodao polje za unos fps vrednosti

## Beleske

- MIDI fajl moze da sadrzi marker ako se sacuva MetaMessage('marker', text='komentar', tick=vreme)
- Pro Tools ima zakucanu vrednost ticks per beat na 960
- [link](http://midi.teragonaudio.com/) odlican izvor midi
- MIDI cuva tickove od prethodne note, a ne od pocetka trake

60bpm 960tpb 4/4
prvi marker 12.000 4 1 000        17*960      16320
drugi marker 33.319 9 2 307       38*960+307  36787
treci 78.159 20 3 153             83*960+153   79833
py    pro tools
11520 16320     4800
20467 36787     16320
43046 79833     36787


- AddMarkers.txt - polomljena implementacija koja pokusava da zapise markere u AAF fajl [link](https://github.com/markreidvfx/pyaaf2/issues/78)

Potreban nam je medium kojim cemo preneti markere u Pro Tools.
Potencijalni fajlovi
- AAF fajl - nije jednostavno
- PTX fajl - proprietary format
  - Tool koji cita ptx fajl [link](https://github.com/zamaudio/ptformat)
- MIDI fajl - [post koji kaze da moze](https://www.reddit.com/r/protools/comments/knrd6b/possible_to_import_markers_from_a_wav_file/) pise u komentaru, probaj sa midijem koji ima meta event sa tekstom
  - [Source](http://duc.avid.com/showthread.php?t=367261) Midi moze da cuva markere.
- BWF fajl - chatgpt kaze da BWF sadrze metadatu u iXML formatu
- AIFF fajl
- neki CUE sheets koji su metadata za neke fajlove
- XMP - metadata fajlovi koji se embeduju u WAV, MP3, AIFF

ChatGPT ideje
- mp3 sa ID3 tagovima
- FLAC podrzava CUE sheets

Zanimljiva diskusija [link](https://forum.blackmagicdesign.com/viewtopic.php?f=33&t=108918)

[Source](https://www.sounddevices.com/software-applications-supporting-markers-and-cues/) Marker information in Pro Tools is not associated with the WAV file. Cues/markers are stored in a separate database file. Pro Tools markers are time-position markers that point to a position in the time-line not a position in a sound file.

- [x] Probaj da dodas marker u session u pro toolsu i sacuvaj projekat i vidi u everythingu koji su se fajlovi promenili da pokusamo da lociramo bazu sa markerima
  - previse izmena u hex editoru. tesko da se skapira
  - [x] radiff2 -x da vidimo bin razliku medju ptx fajlovima

```
// output i frame info
ffprobe -select_streams v -show_frames -show_entries frame=pkt_pts_time,pict_type -of csv C:\Users\Marko\1_Projects\scenedetector\bele_rade_1080.mp4 | findstr /R /C:",I"

```

radiff2 komanda

```
radiff2 -x C:\Users\Marko\1_Projects\scenedetector\ptx_
prazniji\marker1.ptx C:\Users\Marko\1_Projects\scenedetector\ptx_prazniji\marker2.ptx
```