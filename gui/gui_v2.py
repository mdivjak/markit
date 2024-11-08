import tkinter as tk
from tkinter import filedialog, messagebox
from tkinter import ttk
import os
from scenedetect import VideoManager, SceneManager
from scenedetect.detectors import ContentDetector
import mido
from mido import MetaMessage, MidiFile, MidiTrack

def detect_scene_changes(video_path):
    # Initialize video manager and scene manager
    video_manager = VideoManager([video_path])
    scene_manager = SceneManager()

    # Add ContentDetector (content-based scene detection algorithm)
    scene_manager.add_detector(ContentDetector())

    # Start the video manager
    video_manager.start()

    # Perform scene detection
    scene_manager.detect_scenes(frame_source=video_manager)

    # Get list of scene boundaries
    scene_list = scene_manager.get_scene_list()

    # Extract frame numbers
    frame_numbers = [scene[0].get_frames() for scene in scene_list]

    # Release resources
    video_manager.release()

    # Return list of frame numbers
    return frame_numbers

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

def select_video():
    video_path = filedialog.askopenfilename(filetypes=[("Video files", "*.mp4;*.avi;*.mov")])
    if video_path:
        video_path_entry.delete(0, tk.END)
        video_path_entry.insert(0, video_path)

def select_output():
    output_path = filedialog.askdirectory()
    if output_path:
        output_path_entry.delete(0, tk.END)
        output_path_entry.insert(0, output_path)

def process_video():
    video_path = video_path_entry.get()
    output_path = output_path_entry.get()
    midi_file_name = midi_file_name_entry.get()
    
    if not video_path or not output_path or not midi_file_name:
        messagebox.showerror("Error", "Please select video file, output path, and enter MIDI file name.")
        return
    
    output_file = os.path.join(output_path, midi_file_name + ".mid")
    
    print(f"Detecting scene changes in '{video_path}'...")
    frame_numbers = detect_scene_changes(video_path)
    print(f"Finished detecting {len(frame_numbers)} scenes in '{video_path}'.")
    
    print(f"Creating MIDI file with markers for '{video_path}'...")
    create_midi_with_markers(frame_numbers, output_file)
    print(f"MIDI file saved to '{output_file}'")
    messagebox.showinfo("Success", f"MIDI file saved to '{output_file}'")


# Create the main window
root = tk.Tk()
root.title("MarkIt")

# Set the custom icon
# root.iconbitmap('app_icon.ico')

# Apply ttkbootstrap style
style = ttk.Style()
# style.theme_use('cosmo')  # You can choose from various themes like 'cosmo', 'flatly', 'darkly', etc.
# style.master = root

# Video path selection
style.Label(root, text="Video File:").grid(row=0, column=0, padx=10, pady=10)
video_path_entry = style.Entry(root, width=50)
video_path_entry.grid(row=0, column=1, padx=10, pady=10)
style.Button(root, text="Browse", command=select_video).grid(row=0, column=2, padx=10, pady=10)

# Output path selection
style.Label(root, text="Output Path:").grid(row=1, column=0, padx=10, pady=10)
output_path_entry = style.Entry(root, width=50)
output_path_entry.grid(row=1, column=1, padx=10, pady=10)
style.Button(root, text="Browse", command=select_output).grid(row=1, column=2, padx=10, pady=10)

# MIDI file name input
style.Label(root, text="MIDI File Name (without .mid extension):").grid(row=2, column=0, padx=10, pady=10)
midi_file_name_entry = style.Entry(root, width=50)
midi_file_name_entry.grid(row=2, column=1, padx=10, pady=10)

# Process button
style.Button(root, text="Process", command=process_video).grid(row=3, column=0, columnspan=3, pady=20)

# Run the application
root.mainloop()