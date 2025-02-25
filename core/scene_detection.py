# scene_detection.py
from scenedetect import ContentDetector, detect

def detect_scene_changes(video_path):
    # Get list of scene boundaries
    scene_list = detect(video_path, ContentDetector())

    # Extract frame numbers
    frame_numbers = [scene[0].get_frames() for scene in scene_list]
    
    return frame_numbers