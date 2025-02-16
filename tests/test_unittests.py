import os

import mido
from mido import MidiFile
from core.scene_detection import detect_scene_changes
from core.create_midi_with_markers import create_midi_with_markers
from core.get_video_fps import get_video_fps

def test_get_video_fps():
    # Path to the test video file
    test_video_path = 'test_files/bele_rade_1080.mp4'
    
    # Expected FPS of the test video
    expected_fps = 25  # Replace with actual expected FPS
    
    # Call the function to get the FPS of the video
    fps = get_video_fps(test_video_path)
    
    # Assert that the detected FPS matches the expected FPS
    assert fps == expected_fps

def test_scene_detection():
    # Path to the test video file
    test_video_path = 'test_files/bele_rade_1080.mp4'
    
    # Expected frame numbers for scene changes
    expected_frame_numbers = [0, 300, 533, 1121, 1778]  # Replace with actual expected frame numbers
    
    # Call the function to detect scene changes
    frame_numbers = detect_scene_changes(test_video_path)
    
    # Assert that the detected frame numbers match the expected frame numbers
    assert frame_numbers == expected_frame_numbers

def test_create_midi(tmpdir):
    # Define test input
    frame_numbers = [0, 300, 533, 1121, 1778]
    output_filename = os.path.join(tmpdir, 'test_output.mid')
    fps = 25
    bpm = 60

    # Call the function to create the MIDI file
    create_midi_with_markers(frame_numbers, output_filename, fps, bpm)

    # Verify that the MIDI file was created
    assert os.path.exists(output_filename)

    # Load the MIDI file and verify its contents
    mid = MidiFile(output_filename)
    track = mid.tracks[0]

    # Verify the tempo
    msg = track[0]
    assert msg.type == 'set_tempo' and msg.tempo == 1000000 and msg.time == 0

    # Verify time signature
    msg = track[1]
    assert msg.type == 'time_signature' and msg.numerator == 4 and msg.denominator == 4 and msg.time == 0 and msg.clocks_per_click == 24 and msg.notated_32nd_notes_per_beat == 8

    # Verify the marker texts
    marker_texts = [msg.text for msg in track if msg.type == 'marker']
    expected_marker_texts = [f'SC {i + 1}' for i in range(len(frame_numbers))]
    assert marker_texts == expected_marker_texts

    # Verify the marker times
    marker_times = [msg.time for msg in track if msg.type == 'marker']
    expected_marker_times = [0, 11520, 8947, 22579, 25229]
    assert marker_times == expected_marker_times