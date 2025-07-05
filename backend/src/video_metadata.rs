use anyhow::Ok;
use tokio::process::Command;
use serde_json::Value;

pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
}

pub async fn get_video_metadata(video_path: &str) -> Result<VideoMetadata, anyhow::Error> {
    println!("starting video metadata");
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_streams")
        .arg(video_path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute ffprobe: {}", e));

    // println!("{:?}", output);

    // if !output.status.success() {
    //     return Err("ffprobe command failed".to_string());
    // }

    let json_res= String::from_utf8(output.unwrap().stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e));

    
    
    let json_str = json_res.unwrap();
    // println!("json_str is {}", json_str);

    let json: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e)).unwrap();
    
    // println!("json is {}", json);

    let streams = json.get("streams")
        .and_then(|s| s.as_array())
        .ok_or_else(|| "Invalid JSON format".to_string()).unwrap();

    let video_stream = streams.iter()
        .find(|s| s.get("codec_type").and_then(|t| t.as_str()) == Some("video"))
        .ok_or_else(|| "No video stream found".to_string()).unwrap();

    let width = video_stream.get("width")
        .and_then(|w| w.as_u64())
        .ok_or_else(|| "Width not found".to_string()).unwrap() as u32;

    let height = video_stream.get("height")
        .and_then(|h| h.as_u64())
        .ok_or_else(|| "Height not found".to_string()).unwrap() as u32;

    let duration = video_stream.get("duration")
        .and_then(|d| d.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(VideoMetadata {
        width,
        height,
        duration,
    })
}
