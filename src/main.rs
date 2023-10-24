use std::error::Error;
use std::process::Command;
use std::time::Duration;
use std::{fs, thread};

use rppal::pwm::{Channel, Polarity, Pwm};
use tokio::time;

const PWM_FREQ: f64 = 25000.0;
const MIN_TEMP: f32 = 40.0;
const MAX_TEMP: f32 = 70.0;
const FAN_LOW: f32 = 10.0;
const FAN_HIGH: f32 = 100.0;
const FAN_OFF: f32 = 0.0;
const FAN_MAX: f32 = 100.0;
const FAN_GAIN: f32 = (FAN_HIGH - FAN_LOW) / (MAX_TEMP - MIN_TEMP);

fn get_cpu_temp() -> f32 {
    let cpu_temp_content = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .expect("Failed to read CPU temp");
    let cpu_temp_1000_f32 = cpu_temp_content
        .trim_end()
        .parse::<f32>()
        .expect("Could not convert CPU temp to f32");
    let cpu_temp = cpu_temp_1000_f32 / 1000.0;
    return cpu_temp;
}

fn get_gpu_temp() -> f32 {
    let gpu_temp_output = Command::new("vcgencmd")
        .arg("measure_temp")
        .output()
        .expect("Failed to execute command");
    let gpu_temp_str_splitted: Vec<&str> = std::str::from_utf8(&gpu_temp_output.stdout)
        .ok()
        .expect("Failed to convert from byte string")
        .split(['=', '\''].as_ref())
        .collect();
    let gpu_temp = gpu_temp_str_splitted[1];
    return gpu_temp
        .parse::<f32>()
        .expect("Could not convert GPU temp to f32");
}

async fn task_temp() {
    let cpu_temp = get_cpu_temp();
    println!("| CPU temperature: {:.1}\u{00B0} °C |", cpu_temp);
    let delta = cpu_temp - MIN_TEMP;
    let speed = FAN_LOW + (delta.round() * FAN_GAIN);
    println!("{} %", speed);
    if cpu_temp < MIN_TEMP {
        print!("stop")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut interval = time::interval(time::Duration::from_secs(2));
    for _i in 0..5 {
        interval.tick().await;
        task_temp().await;
    }

    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 25 kHz (noctua) with a 25% duty cycle.
    let pwm = Pwm::with_frequency(Channel::Pwm0, PWM_FREQ, 0.25, Polarity::Normal, true)?;
    thread::sleep(Duration::from_secs(2));

    // Reconfigure the PWM channel for an 25 kHz frequency, 50% duty cycle.
    pwm.set_frequency(PWM_FREQ, 0.5)?;
    thread::sleep(Duration::from_secs(30));
    Ok(())

    // When the pwm variable goes out of scope, the PWM channel is automatically disabled.
    // You can manually disable the channel by calling the Pwm::disable() method.
}
