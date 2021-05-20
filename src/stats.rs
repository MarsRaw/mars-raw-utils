

pub fn radians(d:f64) -> f64 {
    d * std::f64::consts::PI / 180.0
}

pub fn degrees(r:f64) -> f64 {
    r * 180.0 / std::f64::consts::PI
}

//https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
pub fn mean(data: &[f32]) -> Option<f32> {
    let sum = data.iter().sum::<f32>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}



pub fn std_deviation(data: &[f32]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data.iter().map(|value| {
                let diff = data_mean - (*value as f32);

                diff * diff
            }).sum::<f32>() / count as f32;

            Some(variance.sqrt())
        },
        _ => None
    }
}


pub fn z_score(pixel_value:f32, data:&[f32]) -> Option<f32> {
    let data_mean = mean(&data);
    let data_std_deviation = std_deviation(&data);
    let data_value = pixel_value;

    match (data_mean, data_std_deviation) {
        (Some(mean), Some(std_deviation)) => {
            let diff = data_value as f32 - mean;
            Some(diff / std_deviation)
        },
        _ => None
    }
}