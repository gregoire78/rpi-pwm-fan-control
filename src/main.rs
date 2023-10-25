use rppal::pwm::{Channel, Polarity, Pwm};
use std::error::Error;
use std::fs;
use std::process::exit;
use tokio::time;

const PWM_FREQ: f64 = 25000.0;
const MIN_TEMP: i32 = 40;
const MAX_TEMP: i32 = 50;
const FAN_LOW: f64 = 0.1;
const FAN_HIGH: f64 = 1.0;
const FAN_OFF: f64 = 0.0;
const FAN_GAIN: f64 = (FAN_HIGH - FAN_LOW) / ((MAX_TEMP - MIN_TEMP) as f64);

fn get_cpu_temp() -> i32 {
    let cpu_temp_content = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .expect("Failed to read CPU temp");
    let cpu_temp_1000_f32 = cpu_temp_content
        .trim_end()
        .parse::<f32>()
        .expect("Could not convert CPU temp to f32");
    let cpu_temp = cpu_temp_1000_f32 / 1000.0;
    return cpu_temp as i32;
}

async fn set_fan_speed(pwm: &Pwm) {
    let mut speed = FAN_OFF;
    static mut LAST_SPEED: f64 = 0.0;

    let cpu_temp = get_cpu_temp();
    let delta: f64 = (cpu_temp - MIN_TEMP).into();
    if delta > 0.0 {
        speed = FAN_LOW + (delta * FAN_GAIN);
        speed = (speed * 100.0).round() / 100.0;
    }
    if cpu_temp >= MAX_TEMP {
        speed = 1.0;
    }
    if speed != unsafe { LAST_SPEED } {
        unsafe { LAST_SPEED = speed };
        pwm.set_frequency(PWM_FREQ, speed).expect("error");
    }
    println!("| CPU temperature: {cpu_temp} °C | speed {speed} |");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        println!("STOPPP");
        let _pwm = Pwm::with_frequency(Channel::Pwm0, PWM_FREQ, FAN_OFF, Polarity::Normal, true)
            .expect("error");
        exit(0);
    });

    let pwm =
        Pwm::with_frequency(Channel::Pwm0, PWM_FREQ, 1.0, Polarity::Normal, true).expect("error");
    let mut interval = time::interval(time::Duration::from_secs(2));

    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 25 kHz (noctua) with a 25% duty cycle.
    loop {
        interval.tick().await;
        set_fan_speed(&pwm).await;
    }

    // When the pwm variable goes out of scope, the PWM channel is automatically disabled.
    // You can manually disable the channel by calling the Pwm::disable() method.
}
