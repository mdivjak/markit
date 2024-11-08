from scene_detection import detect_scene_changes
from create_midi_with_markers import create_midi_with_markers

import tkinter as tk
from tkinter import filedialog, messagebox
import os

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

# Video path selection
tk.Label(root, text="Video File:").grid(row=0, column=0, padx=10, pady=10)
video_path_entry = tk.Entry(root, width=50)
video_path_entry.grid(row=0, column=1, padx=10, pady=10)
tk.Button(root, text="Browse", command=select_video).grid(row=0, column=2, padx=10, pady=10)

# Output path selection
tk.Label(root, text="Output Path:").grid(row=1, column=0, padx=10, pady=10)
output_path_entry = tk.Entry(root, width=50)
output_path_entry.grid(row=1, column=1, padx=10, pady=10)
tk.Button(root, text="Browse", command=select_output).grid(row=1, column=2, padx=10, pady=10)

# MIDI file name input
tk.Label(root, text="MIDI File Name (without .mid extension):").grid(row=2, column=0, padx=10, pady=10)
midi_file_name_entry = tk.Entry(root, width=50)
midi_file_name_entry.grid(row=2, column=1, padx=10, pady=10)

# Process button
tk.Button(root, text="Process", command=process_video).grid(row=3, column=0, columnspan=3, pady=20)

# Run the application
root.mainloop()