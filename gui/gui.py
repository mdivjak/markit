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
import logging
import threading

class TextHandler(logging.Handler):
    def __init__(self, text_widget):
        logging.Handler.__init__(self)
        self.text_widget = text_widget

    def emit(self, record):
        msg = self.format(record)
        self.text_widget.config(state=tk.NORMAL)
        self.text_widget.insert(tk.END, msg + "\n")
        self.text_widget.see(tk.END)
        self.text_widget.config(state=tk.DISABLED)
        window.update_idletasks()  # Update the GUI

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

def setup_logging():
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)
    handler = TextHandler(log_text)
    formatter = logging.Formatter('%(asctime)s - %(levelname)s - %(message)s')
    handler.setFormatter(formatter)
    logger.addHandler(handler)

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
        enable_buttons()
        return
    
    logging.info(f"Detecting scene changes in '{video_path}'...")
    logging.info("This may take a few minutes depending on the video duration.")
    frame_numbers = detect_scene_changes(video_path)
    logging.info(f"Finished detecting {len(frame_numbers)} scenes in '{video_path}'.")

    video_fps = get_video_fps(video_path)
    logging.info(f"Video frame rate is {video_fps} FPS.")

    output_file = os.path.join(output_path, midi_file_name + ".mid")
    logging.info(f"Creating MIDI file with markers for '{video_path}'...")
    create_midi_with_markers(frame_numbers, output_file, fps=video_fps)
    logging.info(f"MIDI file saved to '{output_file}'")
    logging.info("Success: MIDI file saved.")
    messagebox.showinfo("Success", "MIDI file saved successfully.")

    # Re-enable the buttons and entries after processing is complete
    enable_buttons()

def process_video_thread():
    # Disable the buttons and entries when the thread starts
    disable_buttons()
    threading.Thread(target=process_video).start()

def disable_buttons():
    browse_video_button.config(state=tk.DISABLED)
    browse_output_button.config(state=tk.DISABLED)
    process_button.config(state=tk.DISABLED)
    video_path_entry.config(state=tk.DISABLED)
    output_path_entry.config(state=tk.DISABLED)
    midi_file_name_entry.config(state=tk.DISABLED)

def enable_buttons():
    browse_video_button.config(state=tk.NORMAL)
    browse_output_button.config(state=tk.NORMAL)
    process_button.config(state=tk.NORMAL)
    video_path_entry.config(state=tk.NORMAL)
    output_path_entry.config(state=tk.NORMAL)
    midi_file_name_entry.config(state=tk.NORMAL)

style = Style(theme='darkly')
window = style.master

window.title("MarkIt")

# Video path selection
ttk.Label(window, text="Video File:").grid(row=0, column=0, padx=10, pady=10)
video_path_entry = ttk.Entry(window, width=50, style='info.TEntry')
video_path_entry.grid(row=0, column=1, padx=10, pady=10)
browse_video_button = ttk.Button(window, text="Browse", command=select_video)
browse_video_button.grid(row=0, column=2, padx=10, pady=10)

# Output path selection
ttk.Label(window, text="Output Path:").grid(row=1, column=0, padx=10, pady=10)
output_path_entry = ttk.Entry(window, width=50, style='info.TEntry')
output_path_entry.grid(row=1, column=1, padx=10, pady=10)
browse_output_button = ttk.Button(window, text="Browse", command=select_output_path)
browse_output_button.grid(row=1, column=2, padx=10, pady=10)

# MIDI file name input
ttk.Label(window, text="MIDI File Name (without .mid extension):").grid(row=2, column=0, padx=10, pady=10)
midi_file_name_entry = ttk.Entry(window, width=50, style='info.TEntry')
midi_file_name_entry.grid(row=2, column=1, padx=10, pady=10)

# Process button
process_button = ttk.Button(window, text="Process", command=process_video_thread)
process_button.grid(row=3, column=0, columnspan=3, pady=20)

# Log label
ttk.Label(window, text="Logs:").grid(row=4, column=0, padx=10, pady=10, sticky='w')

# Log text widget
log_text = tk.Text(window, height=10, width=200, state=tk.DISABLED)
log_text.grid(row=5, column=0, columnspan=3, padx=10, pady=10)

# Version number
version_number = "MarkIt Version 0.6"
ttk.Label(window, text=version_number, style='info.TLabel').grid(row=6, column=2, padx=10, pady=10, sticky='e')

# Setup logging
setup_logging()

# Run the application
window.mainloop()