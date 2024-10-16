use anyhow::Result;
use esp_idf_hal::{
    self,
    i2c::{I2cConfig, I2cDriver},
};
use esp_idf_svc::hal::gpio;
use esp_idf_sys::camera as espcam;

mod espcam_wrap {
    use anyhow::{anyhow, Result};
    use esp_idf_hal::{
        gpio::Pin,
        i2c::{I2cConfig, I2cDriver},
        prelude::Peripherals,
    };

    use esp_idf_sys::{
        camera::{
            self, camera_config_t, camera_config_t__bindgen_ty_1, camera_config_t__bindgen_ty_2,
            camera_fb_location_t_CAMERA_FB_IN_DRAM, camera_fb_location_t_CAMERA_FB_IN_PSRAM,
            camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY, esp_camera_init, framesize_t_FRAMESIZE_HD,
            framesize_t_FRAMESIZE_VGA, ledc_channel_t_LEDC_CHANNEL_0, ledc_timer_t_LEDC_TIMER_0,
            pixformat_t_PIXFORMAT_JPEG,
        },
        ESP_OK,
    };

    pub struct Camera<'d> {
        i2c: I2cDriver<'d>,
    }

    impl<'d> Camera<'d> {
        pub fn configure() -> Result<Self> {
            let pf = Peripherals::take().unwrap();

            let i2c_cfg = I2cConfig::new().baudrate(100000.into());
            let i2c = I2cDriver::new(pf.i2c0, pf.pins.gpio13, pf.pins.gpio3, &i2c_cfg)?;

            let ccfg: camera_config_t = camera_config_t {
                pin_pwdn: pf.pins.gpio9.pin(),
                pin_reset: pf.pins.gpio11.pin(),
                pin_xclk: pf.pins.gpio8.pin(),
                pin_d7: pf.pins.gpio12.pin(),
                pin_d6: pf.pins.gpio18.pin(),
                pin_d5: pf.pins.gpio17.pin(),
                pin_d4: pf.pins.gpio15.pin(),
                pin_d3: pf.pins.gpio6.pin(),
                pin_d2: pf.pins.gpio4.pin(),
                pin_d1: pf.pins.gpio5.pin(),
                pin_d0: pf.pins.gpio7.pin(),
                pin_vsync: pf.pins.gpio10.pin(),
                pin_href: pf.pins.gpio40.pin(),
                pin_pclk: pf.pins.gpio16.pin(),
                xclk_freq_hz: 15000000,
                ledc_timer: ledc_timer_t_LEDC_TIMER_0,
                ledc_channel: ledc_channel_t_LEDC_CHANNEL_0,
                pixel_format: pixformat_t_PIXFORMAT_JPEG,
                frame_size: framesize_t_FRAMESIZE_VGA,
                jpeg_quality: 10,
                fb_count: 1,
                fb_location: camera_fb_location_t_CAMERA_FB_IN_DRAM,
                grab_mode: camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,
                sccb_i2c_port: i2c.port() as i32,
                __bindgen_anon_1: camera_config_t__bindgen_ty_1 { pin_sscb_sda: -1 },
                __bindgen_anon_2: camera_config_t__bindgen_ty_2 { pin_sscb_scl: -1 },
            };

            let err = unsafe { esp_camera_init(&ccfg) };

            if err == ESP_OK {
                Ok(Self { i2c })
            } else {
                Err(anyhow!("failed to configure the camera"))
            }
        }
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Intercepting power...");
    log::info!("Initializing and connecting to WiFi...");
    log::info!("Configuring I2C...");
    log::info!("Configuring camera...");
    let camera = espcam_wrap::Camera::configure().unwrap();
    log::info!("Capturing image...");
}
