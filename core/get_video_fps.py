import cv2

def get_video_fps(video_path):
    # Open the video file
    video_capture = cv2.VideoCapture(video_path)
    
    # Get the FPS (frames per second) of the video
    fps = video_capture.get(cv2.CAP_PROP_FPS)
    
    # Release the video capture object
    video_capture.release()
    
    return fps