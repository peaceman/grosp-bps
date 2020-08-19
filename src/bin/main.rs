use hls_m3u8::MediaPlaylist;

fn main() {
    println!("Hello, world!");

    let m3u8 = "#EXTM3U
   #EXT-X-STREAM-INF:BANDWIDTH=1280000,AVERAGE-BANDWIDTH=1000000
   http://example.com/low.m3u8
   #EXT-X-STREAM-INF:BANDWIDTH=2560000,AVERAGE-BANDWIDTH=2000000
   http://example.com/mid.m3u8
   #EXT-X-STREAM-INF:BANDWIDTH=7680000,AVERAGE-BANDWIDTH=6000000
   http://example.com/hi.m3u8
   #EXT-X-STREAM-INF:BANDWIDTH=65000,CODECS=\"mp4a.40.5\"
   http://example.com/audio-only.m3u8";

   let parse_result = m3u8.parse::<MediaPlaylist>();

   if let Err(e) = parse_result {
       println!("error: {}", e);
   }

    assert!(m3u8.parse::<MediaPlaylist>().is_ok());
}
