import sys
from TikTokApi import TikTokApi

api = TikTokApi.get_instance(generate_static_device_id=True)

url = sys.argv[1]
path = sys.argv[2]

with open(path, "wb") as wf:
    video = api.get_video_by_url(url)
    wf.write(video)
