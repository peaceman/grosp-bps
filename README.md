GRoSP - Balancing Playlist Spreader
===================================

Forwards incoming m3u8 playlist requests to a configured upstream base url and changes the outgoing playlist so that the m3u8 playlist consumer downloads segments from a list of edge nodes.

The list of edge nodes that is used for a playlist request is randomized and based on the currently available edge nodes supplied via Consul.

Example
-------
An example with upstream_base_url https://upstream and available edge nodes https://alpha and https://beta

Client Request: `/playlist/live.m3u8`

Upstream Request: `https://upstream/live.m3u8`

Upstream Response:
```
#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:8
#EXT-X-MEDIA-SEQUENCE:2680

#EXTINF:7.975,
https://priv.example.com/fileSequence2680.ts
#EXTINF:7.941,
https://priv.example.com/fileSequence2681.ts
#EXTINF:7.975,
https://priv.example.com/fileSequence2682.ts
```

Response sent to client:
```
#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:8
#EXT-X-MEDIA-SEQUENCE:2680

#EXTINF:7.975,
https://beta/fileSequence2680.ts
#EXTINF:7.941,
https://alpha/fileSequence2681.ts
#EXTINF:7.975,
https://alpha/fileSequence2682.ts
```
