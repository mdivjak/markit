# scene_detection.py
from scenedetect import VideoManager, SceneManager
from scenedetect.detectors import ContentDetector

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
    
    return frame_numbers