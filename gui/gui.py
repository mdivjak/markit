import sys
sys.path.append('../')

from core.scene_detection import detect_scene_changes
from core.create_midi_with_markers import create_midi_with_markers
from core.get_video_fps import get_video_fps

import tkinter as tk
from tkinter import filedialog, messagebox
from tkinter import ttk
from ttkbootstrap import Style

import os

def select_video():
    video_path = filedialog.askopenfilename(filetypes=[("Video files", "*.mp4;*.avi;*.mov")])
    if video_path:
        video_path_entry.delete(0, tk.END)
        video_path_entry.insert(0, video_path)

def select_output_path():
    output_path = filedialog.askdirectory()
    if output_path:
        output_path_entry.delete(0, tk.END)
        output_path_entry.insert(0, output_path)

def log_message(message):
    log_text.config(state=tk.NORMAL)
    log_text.insert(tk.END, message + "\n")
    log_text.see(tk.END)
    log_text.config(state=tk.DISABLED)
    window.update_idletasks()

def process_video():
    # Clear the log text widget
    log_text.config(state=tk.NORMAL)
    log_text.delete(1.0, tk.END)
    log_text.config(state=tk.DISABLED)
    window.update_idletasks()

    video_path = video_path_entry.get()
    output_path = output_path_entry.get()
    midi_file_name = midi_file_name_entry.get()
    
    if not video_path or not output_path or not midi_file_name:
        messagebox.showerror("Error", "Please select video file, output path, and enter MIDI file name.")
        return
    
    video_fps = get_video_fps(video_path)
    output_file = os.path.join(output_path, midi_file_name + ".mid")
    
    log_message(f"Detecting scene changes in '{video_path}'...")
    log_message("This may take a few minutes depending on the video duration.")
    frame_numbers = detect_scene_changes(video_path)
    log_message(f"Finished detecting {len(frame_numbers)} scenes in '{video_path}'.")
    
    log_message(f"Video frame rate is {video_fps} FPS.")

    log_message(f"Creating MIDI file with markers for '{video_path}'...")
    create_midi_with_markers(frame_numbers, output_file, fps=video_fps)
    log_message(f"MIDI file saved to '{output_file}'")
    log_message("Success: MIDI file saved.")
    messagebox.showinfo("Success", "MIDI file saved successfully.")


style = Style(theme='darkly')
window = style.master

window.title("MarkIt")

# Video path selection
ttk.Label(window, text="Video File:").grid(row=0, column=0, padx=10, pady=10)
video_path_entry = ttk.Entry(window, width=50, style='info.TEntry')
video_path_entry.grid(row=0, column=1, padx=10, pady=10)
ttk.Button(window, text="Browse", command=select_video).grid(row=0, column=2, padx=10, pady=10)

# Output path selection
ttk.Label(window, text="Output Path:").grid(row=1, column=0, padx=10, pady=10)
output_path_entry = ttk.Entry(window, width=50, style='info.TEntry')
output_path_entry.grid(row=1, column=1, padx=10, pady=10)
ttk.Button(window, text="Browse", command=select_output_path).grid(row=1, column=2, padx=10, pady=10)

# MIDI file name input
ttk.Label(window, text="MIDI File Name (without .mid extension):").grid(row=2, column=0, padx=10, pady=10)
midi_file_name_entry = ttk.Entry(window, width=50, style='info.TEntry')
midi_file_name_entry.grid(row=2, column=1, padx=10, pady=10)

# Process button
ttk.Button(window, text="Process", command=process_video).grid(row=3, column=0, columnspan=3, pady=20)

# Log label
ttk.Label(window, text="Logs:").grid(row=4, column=0, padx=10, pady=10, sticky='w')

# Log text widget
log_text = tk.Text(window, height=10, width=200, state=tk.DISABLED)
log_text.grid(row=5, column=0, columnspan=3, padx=10, pady=10)

# Run the application
window.mainloop()