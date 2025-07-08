use anyhow::{Context, Result};
use std::{path::Path, process::Command};
use std::io;
// mod video_metadata;
use reqwest;
use ollama_rs::models::ModelOptions;

use std::path::PathBuf;
// use video_metadata::get_video_metadata;
use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};
/// Supported transition types for video merging
#[derive(Debug, Clone, Copy)]
pub enum TransitionType {
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    Distance,
    FadeBlack,
    FadeWhite,
    RectCrop,
    CircleOpen,
    CircleClose,
    Dissolve,
    Pixelize,
    HBlur,
    WipeTL,
    WipeTR,
    WipeBL,
    WipeBR,
}

pub enum LUTs {
    PictureFXLeicaM8BW125,
    RetroWarm,
}

/// Concatenates two videos together without any transition effects
///
/// # Arguments
/// * `input1` - Path to the first input video
/// * `input2` - Path to the second input video
/// * `output` - Path where the output video will be saved
pub async fn concatenate_videos(
    input1: &str,
    input2: &str,
    output: &str,
) -> Result<()> {
    if !Path::new(input1).exists() {
        anyhow::bail!("First input video not found: {}", input1);
    }
    if !Path::new(input2).exists() {
        anyhow::bail!("Second input video not found: {}", input2);
    }

    // Use demuxer 
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input1)
        .arg("-i")
        .arg(input2)
        .arg("-filter_complex")
        .arg("[0:v][0:a][1:v][1:a]concat=n=2:v=1:a=1")
        .arg("-c:v")
        .arg("libx264")
        .arg("-c:a")
        .arg("aac")
        .arg("-y")
        .arg(output)
        .status()
        .context("Failed to execute FFmpeg for video concatenation")?;

    if !status.success() {
        anyhow::bail!("Failed to concatenate videos");
    }

    Ok(())
}


async fn add_centered_text_to_video(
    input_path: &str,
    output_path: &str,
    font_path: &str,
    text: &str,
    font_size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(input_path).exists() {
        return Err(format!("Input file not found: {}", input_path).into());
    }

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(format!(
            "drawtext=fontfile={}:text='{}':fontcolor=white:fontsize={}:x=(w-text_w)/2:y=(h-text_h)/2:box=1:boxcolor=black@0.5:boxborderw=5",
            font_path, text, font_size
        ))
        .arg("-codec:a")
        .arg("copy")
        .arg("-y")  
        .arg(output_path)
        .status()?;

    if status.success() {
        println!("Successfully created video with text at: {}", output_path);
        Ok(())
    } else {
        Err(format!("FFmpeg command failed with status: {}", status).into())
    }
}

/// Splits a video file into two parts at the specified time.
///
/// # Arguments
/// * `input_path` - Path to the input video file
/// * `output_path1` - Path where the first part (before split time) will be saved
/// * `output_path2` - Path where the second part (after split time) will be saved
/// * `split_time` - Time in seconds where to split the video
///
pub async fn split_video(
    input_path: &str,
    output_path1: &str,
    output_path2: &str,
    split_time: f64,
) -> Result<()> {
    // Check if input file exists
    if !Path::new(input_path).exists() {
        anyhow::bail!("Input file not found: {}", input_path);
    }

    // First part: from start to split_time
    let status1 = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-t")
        .arg(split_time.to_string())
        .arg("-c")
        .arg("copy")
        .arg("-y") // Overwrite if exists
        .arg(output_path1)
        .status()
        .context("Failed to execute FFmpeg for first part")?;

    if !status1.success() {
        anyhow::bail!("Failed to create first part of the video");
    }

    // Second part: from split_time to end
    let status2 = Command::new("ffmpeg")
        .arg("-ss")
        .arg(split_time.to_string())
        .arg("-i")
        .arg(input_path)
        .arg("-c")
        .arg("copy")
        .arg("-y") // Overwrite if exists
        .arg(output_path2)
        .status()
        .context("Failed to execute FFmpeg for second part")?;

    if !status2.success() {
        anyhow::bail!("Failed to create second part of the video");
    }

    Ok(())
}

/// Merges two videos with a transition effect between them
///
/// # Arguments
/// * `input1` - Path to the first input video
/// * `input2` - Path to the second input video
/// * `output` - Path where the output video will be saved
/// * `transition_type` - Type of transition to use
/// * `duration` - Duration of the transition in seconds
/// * `offset` - Optional offset in seconds from the end of the first video where transition should start
pub async fn merge_videos_with_transition(
    input1: &str,
    input2: &str,
    output: &str,
    transition_type: TransitionType,
    duration: f32,
    offset: Option<f32>,
) -> Result<()> {
    // Check if input files exist
    if !Path::new(input1).exists() {
        anyhow::bail!("First input file not found: {}", input1);
    }
    if !Path::new(input2).exists() {
        anyhow::bail!("Second input file not found: {}", input2);
    }

    // Map transition type to xfade parameter
    let transition_name = match transition_type {
        TransitionType::Fade => "fade",
        TransitionType::SlideLeft => "slideleft",
        TransitionType::SlideRight => "slideright",
        TransitionType::SlideUp => "slideup",
        TransitionType::SlideDown => "slidedown",
        TransitionType::WipeLeft => "wipeleft",
        TransitionType::WipeRight => "wiperight",
        TransitionType::WipeUp => "wipeup",
        TransitionType::WipeDown => "wipedown",
        TransitionType::Distance => "distance",
        TransitionType::FadeBlack => "fadeblack",
        TransitionType::FadeWhite => "fadewhite",
        TransitionType::RectCrop => "rectcrop",
        TransitionType::CircleOpen => "circleopen",
        TransitionType::CircleClose => "circleclose",
        TransitionType::Dissolve => "pixelize",
        TransitionType::Pixelize => "hblur",
        TransitionType::HBlur => "wipetl",
        TransitionType::WipeTL => "wipetl",
        TransitionType::WipeTR => "wipetr",
        TransitionType::WipeBL => "wipebl",
        TransitionType::WipeBR => "wipebr",
    };

    // Calculate the transition offset (default to 1 second before the end of the first video)
    let offset = offset.unwrap_or_else(|| {
        // Get duration of first video to set default offset
        let duration_cmd = Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", input1])
            .output()
            .ok()
            .and_then(|output| {
                String::from_utf8(output.stdout).ok()
                    .and_then(|s| s.trim().parse::<f32>().ok())
            });
        
        // Default to 1 second before end if we can't determine duration
        duration_cmd.map_or(1.0, |d| (d - 1.0).max(0.0))
    });

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input1)
        .arg("-i")
        .arg(input2)
        .arg("-filter_complex")
        .arg(format!(
            "[0:v] [1:v] xfade=transition={}:duration={}:offset={} [v];[0:a][1:a] acrossfade=d={} [a]",
            transition_name, duration, offset, duration
        ))
        .arg("-map")
        .arg("[v]")  // Map the video stream
        .arg("-map")
        .arg("[a]")  // Map the audio stream
        .arg("-c:v")
        .arg("libx264")  // Video codec
        .arg("-c:a")
        .arg("aac")      // Audio codec
        .arg("-b:a")
        .arg("192k")     // Audio bitrate
        .arg("-y")  // Overwrite output file if it exists
        .arg(output)
        .status()
        .context("Failed to execute FFmpeg for video merging")?;

    if !status.success() {
        anyhow::bail!("Failed to merge videos with transition");
    }

    Ok(())
}

/// Overlays an image on a video
///
/// # Arguments
/// * `input_video` - Path to the input video
/// * `output_video` - Path where the output video will be saved
/// * `image_path` - Path to the image to overlay
/// * `x` - X-coordinate of the top-left corner of the image
/// * `y` - Y-coordinate of the top-left corner of the image
/// * `opacity` - Opacity of the image (between 0 and 1)
pub async fn overlay_image_on_video(
    input_video: &str,
    output_video: &str,
    image_path: &str,
    x: i32,
    y: i32,
    opacity: f32,
    width: Option<i32>,
    height: Option<i32>,
) -> Result<()> {
    // Check if input files exist
    if !Path::new(input_video).exists() {
        anyhow::bail!("Input video not found: {}", input_video);
    }
    if !Path::new(image_path).exists() {
        anyhow::bail!("Image not found: {}", image_path);
    }

    // Use FFmpeg overlay filter to overlay the image
    let scale_filter = match (width, height) {
        (Some(w), Some(h)) => format!("scale={}:{}", w, h),
        (Some(w), None) => format!("scale={}:ih*{}*sar", w, w),
        (None, Some(h)) => format!("scale=iw*{}*dar:{}", h, h),
        (None, None) => "scale=1080:1080".to_string(),
    };

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_video)
        .arg("-i")
        .arg(image_path)
        .arg("-filter_complex")
        .arg(format!(
            "[1:v]format=rgba,{},setsar=1[logo];[0:v][logo]overlay=x={}:y={}:format=auto,format=yuv420p,setsar=1",
            scale_filter, x, y
        ))
        .arg("-c:v")
        .arg("libx264")
        .arg("-c:a")
        .arg("copy")
        .arg("-y")
        .arg(output_video)
        .status()
        .context("Failed to overlay image on video")?;

    if !status.success() {
        anyhow::bail!("Failed to overlay image on video");
    }

    Ok(())
}

/// Creates a solid color image using FFmpeg
///
/// # Arguments
/// * `color` - Color specification (e.g., "red", "#FF0000", "rgb(255,0,0)")
/// * `dimensions` - Tuple of (width, height) in pixels
/// * `output_path` - Path where the output image will be saved
///
/// # Example
/// ```
/// create_solid_color_image("red", (800, 600), "red_square.png").await.unwrap();
/// ```
async fn create_solid_color_image(color: &str, dimensions: (u32, u32), output_path: &str) -> Result<()> {
    let (width, height) = dimensions;
    let size = format!("{}x{}", width, height);
    
    let output = Command::new("ffmpeg")
        .args([
            "-f", "lavfi",
            "-i",
            &format!("color=c={}:s={}", color, size),
            "-frames:v", "1",
            "-update", "1",
            output_path,
        ])
        .output()
        .context("Failed to execute ffmpeg command")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "FFmpeg failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Applies a LUT (Look-Up Table) to a video file using ffmpeg.
///
/// # Arguments
/// * `input_path` - Path to the input video file
/// * `output_path` - Path where the processed video will be saved
/// * `lut_path` - Path to the LUT file (.cube format)
///
/// # Returns
/// * `Result<()>` - Ok(()) on success, or an error if the process fails
///
/// # Example
/// ```
/// apply_lut_to_video("input.mp4", "output.mp4", "my_lut.cube").await?;
/// ```
pub async fn apply_lut_to_video(
    input_path: &str,
    output_path: &str,
    lut: LUTs,
) -> Result<()> {

    // Verify input file exists
    if !Path::new(input_path).exists() {
        anyhow::bail!("Input video not found: {}", input_path);
    }
    
    let lut_path = match lut {
        LUTs::PictureFXLeicaM8BW125 => "PictureFX-LeicaM8-BW-125.cube",
        LUTs::RetroWarm => "Retro-Warm.cube",
    };
    // Verify LUT file exists
    if !Path::new(&lut_path).exists() {
        anyhow::bail!("LUT file not found: {}", &lut_path);
    }
    // Build the ffmpeg command
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", input_path,
            "-vf", &format!("lut3d={}", lut_path),
            "-c:a", "copy",  // Copy audio without re-encoding
            output_path,
            "-y",  // Overwrite output file if it exists
        ])
        .status()
        .context("Failed to execute ffmpeg command")?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("ffmpeg process failed with status: {}", status);
    }
}

/// Get the CPU temperature in Celsius.
#[ollama_rs::function]
async fn get_cpu_temperature() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok("42.7".to_string())
}

/// Get the available space in bytes for a given path.
///
/// * path - Path to check available space for.
#[ollama_rs::function]
async fn get_available_space(
    path: PathBuf,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok("30".to_string())
}

/// Get the weather for a given city.
///
/// * city - City to get the weather for.
#[ollama_rs::function]
async fn get_weather(city: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    Ok(reqwest::get(format!("https://wttr.in/{city}?format=%C+%t"))
        .await?
        .text()
        .await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Add text to video
    let input_path = "src/insert.mp4";
    let font_path = "src/Roboto_Mono/RobotoMono-Italic-VariableFont_wght.ttf";
    let text = "Im drinking coffee";
    let font_size = 50;
    // let metadata_res = video_metadata::get_video_metadata("src/input.mp4").await;
    // let metadata = metadata_res.unwrap();
    // let width = metadata.width;
    // let hight = metadata.height;
    // let duration = metadata.duration;
    // println!("width: {}, hight: {}, duration: {},", width, hight, duration);

    // let black_img = create_solid_color_image("black", (800, 50), "black_square.png").await.unwrap();
    // overlay_image_on_video("src/input.mp4", "with_image.mp4", "black_square.png", ((width / 2) - 400) as i32, (hight - 200) as i32, 0.0, Some(1000), Some(100)).await.unwrap();


    
    // if let Err(e) = split_video("with_image.mp4", "temp1.mp4", "temp2.mp4", 5.0).await {
    //     eprintln!("Error splitting video: {}", e);
    //     std::process::exit(1);
    // }
    // add_centered_text_to_video("temp2.mp4", "temp2-text.mp4", font_path, text, font_size).await?;
    // concatenate_videos("temp1.mp4", "temp2-text.mp4", "final-out.mp4").await;

    // apply_lut_to_video("lut-test1.webm", "lut-test1.2-lut.mp4", LUTs::RetroWarm).await.unwrap();

    let ollama = Ollama::default();
    let history = vec![];
    let mut coordinator = Coordinator::new(ollama, "llama3.2".to_string(), history)
        .add_tool(get_cpu_temperature);

    let user_messages = vec![
        "What's the CPU temperature?",
        "What's the available space in the root directory?",
        "What's the weather in Berlin?",
    ];

    for user_message in user_messages {
        println!("User: {user_message}");

        let user_message = ChatMessage::user(user_message.to_owned());
        let resp = coordinator.chat(vec![user_message]).await?;
        println!("Assistant: {}", resp.message.content);
    }

    Ok(())
}