use std::ffi::c_void;

use anyhow::{anyhow, Result};
use esp_idf_hal::{
    gpio::{AnyIOPin, InputPin, OutputPin, Pin, Pins},
    i2c::{self, I2cConfig, I2cDriver},
    peripheral::Peripheral,
    prelude::Peripherals,
};

use esp_idf_sys::{
    camera::{
        camera_config_t, camera_config_t__bindgen_ty_1, camera_config_t__bindgen_ty_2,
        camera_fb_location_t_CAMERA_FB_IN_DRAM, camera_fb_t,
        camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY, esp_camera_fb_get, esp_camera_fb_return,
        esp_camera_init, framesize_t_FRAMESIZE_VGA, ledc_channel_t_LEDC_CHANNEL_0,
        ledc_timer_t_LEDC_TIMER_0, pixformat_t_PIXFORMAT_JPEG,
    },
    ESP_OK,
};

pub struct CameraPeriphs<I> {
    pub i2c: I,
    pub sda: AnyIOPin,
    pub sdl: AnyIOPin,
    pub pin_pwdn: AnyIOPin,
    pub pin_reset: AnyIOPin,
    pub pin_xclk: AnyIOPin,
    pub pin_d7: AnyIOPin,
    pub pin_d6: AnyIOPin,
    pub pin_d5: AnyIOPin,
    pub pin_d4: AnyIOPin,
    pub pin_d3: AnyIOPin,
    pub pin_d2: AnyIOPin,
    pub pin_d1: AnyIOPin,
    pub pin_d0: AnyIOPin,
    pub pin_vsync: AnyIOPin,
    pub pin_href: AnyIOPin,
    pub pin_pclk: AnyIOPin,
}

pub struct Camera<'d> {
    _i2c: I2cDriver<'d>,
}

impl<'d> Camera<'d> {
    pub fn configure<I>(cpf: CameraPeriphs<I>) -> Result<Self>
    where
        I: Peripheral + 'd,
        I::P: i2c::I2c,
    {
        let i2c_cfg = I2cConfig::new().baudrate(100000.into());
        let i2c = I2cDriver::new(cpf.i2c, cpf.sda, cpf.sdl, &i2c_cfg)?;

        let ccfg: camera_config_t = camera_config_t {
            pin_pwdn: cpf.pin_pwdn.pin(),
            pin_reset: cpf.pin_reset.pin(),
            pin_xclk: cpf.pin_xclk.pin(),
            pin_d7: cpf.pin_d7.pin(),
            pin_d6: cpf.pin_d6.pin(),
            pin_d5: cpf.pin_d5.pin(),
            pin_d4: cpf.pin_d4.pin(),
            pin_d3: cpf.pin_d3.pin(),
            pin_d2: cpf.pin_d2.pin(),
            pin_d1: cpf.pin_d1.pin(),
            pin_d0: cpf.pin_d0.pin(),
            pin_vsync: cpf.pin_vsync.pin(),
            pin_href: cpf.pin_href.pin(),
            pin_pclk: cpf.pin_pclk.pin(),
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
            Ok(Self { _i2c: i2c })
        } else {
            Err(anyhow!("failed to configure the camera"))
        }
    }

    pub fn get_data(&self) -> Result<Fb> {
        unsafe {
            let fb_ptr = esp_camera_fb_get();
            // TODO: wtf
            if fb_ptr as *mut c_void == (0 as *mut c_void) {
                Err(anyhow!("failed to get the framebuffer"))
            } else {
                Ok(Fb::new(fb_ptr))
            }
        }
    }
}

pub struct Fb {
    fb: *mut camera_fb_t,
}

impl Fb {
    fn new(fb: *mut camera_fb_t) -> Self {
        Self { fb }
    }

    pub fn slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((*self.fb).buf, (*self.fb).len) }
    }
}

impl Drop for Fb {
    fn drop(&mut self) {
        unsafe {
            esp_camera_fb_return(self.fb);
        }
    }
}
