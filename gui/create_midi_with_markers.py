import mido
from mido import MetaMessage, MidiFile, MidiTrack

def create_midi_with_markers(frame_numbers, output_filename, fps=25, bpm=60):
    # Video properties
    ticks_per_beat = 960  # Pro Tools uses 960 ticks per beat
    ticks_per_frame = ticks_per_beat / fps  # Ticks per frame

    # Create a new MIDI file and a single track
    mid = MidiFile(ticks_per_beat=ticks_per_beat)
    track = MidiTrack()
    mid.tracks.append(track)

    # Convert BPM to tempo
    tempo = mido.bpm2tempo(bpm)

    # Add tempo and time signature to the track
    track.append(MetaMessage('set_tempo', tempo=tempo))
    track.append(MetaMessage('time_signature', numerator=4, denominator=4, clocks_per_click=24, notated_32nd_notes_per_beat=8, time=0))

    # Add markers at each scene change frame number
    previous_tick = 0
    for i, frame in enumerate(frame_numbers):
        # Calculate the tick difference from the previous frame
        if i == 0:
            tick_diff = 0
        else:
            tick_diff = int((frame - frame_numbers[i - 1]) * ticks_per_frame)
        
        marker_text = f'SC {i + 1}'  # Custom marker text for each scene
        
        # Add a marker (meta event) at the scene change time
        track.append(MetaMessage('marker', text=marker_text, time=tick_diff))
        previous_tick += tick_diff

    # Save the MIDI file with the provided name
    mid.save(output_filename)
    print(f"MIDI file '{output_filename}' created successfully.")