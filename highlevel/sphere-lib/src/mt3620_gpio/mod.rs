extern crate sphere_sys;
use sphere_sys::GPIO_OpenAsOutput;
use sphere_sys::GPIO_SetValue;

const GPIO_OUTPUT_MODE_PUSH_PULL: u8 = 0;
const GPIO_VALUE_LOW: u8 = 0;
const GPIO_VALUE_HIGH: u8 = 1;

pub struct GpioPort {
    fd: i32,
}

impl GpioPort {
    pub fn open(number: i32) -> GpioPort {
        let out_fd =
            unsafe { GPIO_OpenAsOutput(number, GPIO_OUTPUT_MODE_PUSH_PULL, GPIO_VALUE_HIGH) };

        GpioPort { fd: out_fd }
    }

    pub fn set_high(&self) {
        unsafe {
            GPIO_SetValue(self.fd, GPIO_VALUE_HIGH);
        }
    }

    pub fn set_low(&self) {
        unsafe {
            GPIO_SetValue(self.fd, GPIO_VALUE_LOW);
        }
    }

    pub fn set(&self, state: bool) {
        match state {
            true => self.set_high(),
            false => self.set_low(),
        }
    }
}
