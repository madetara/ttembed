import sys
from TikTokApi import TikTokApi

api = TikTokApi(generate_static_device_id=True)

url = sys.argv[1]
path = sys.argv[2]

with open(path, "wb") as wf:
    video = api.video(url=url).bytes()
    wf.write(video)
