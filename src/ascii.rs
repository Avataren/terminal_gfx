pub fn angle_to_ascii(angle: f32) -> char {
    let angle_deg = angle.to_degrees();
    match angle_deg {
        a if (-22.5..22.5).contains(&a) => '|',
        a if (22.5..67.5).contains(&a) => '/',
        a if (67.5..112.5).contains(&a) => '-',
        a if (112.5..157.5).contains(&a) => '\\',
        a if (157.5..=180.0).contains(&a) || (-180.0..-157.5).contains(&a) => '|',
        a if (-157.5..-112.5).contains(&a) => '/',
        a if (-112.5..-67.5).contains(&a) => '-',
        a if (-67.5..-22.5).contains(&a) => '\\',
        _ => '+',
    }
}

pub fn brightness_to_ascii(brightness: u8, invert: bool) -> char {
    const ASCII_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
    
    // Apply gamma correction (gamma = 2.2)
    let corrected_brightness = (brightness as f32 / 255.0).powf(1.0 / 2.2);
    
    // Invert if needed
    let normalized_brightness = if invert {
        1.0 - corrected_brightness
    } else {
        corrected_brightness
    };
    
    // Map to ASCII character
    let index = (normalized_brightness * (ASCII_CHARS.len() - 1) as f32).round() as usize;
    ASCII_CHARS[index]
}