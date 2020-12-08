#![allow(non_camel_case_types)]

extern crate sphere_sys;

use sphere_sys::*;

use alloc::boxed::Box;

use core::mem::MaybeUninit;

#[derive(Copy, Clone)]
pub struct Watchdog {
    timer: *mut timer_t,
    timeout: usize,
}

impl Watchdog {
    pub fn create(seconds: usize) -> Watchdog {
        unsafe {
            let watchdog_timer = Box::<timer_t>::new_uninit().assume_init();
            let watchdog_timer = Box::into_raw(watchdog_timer);

            let watchdog_interval = itimerspec {
                it_interval: timespec {
                    tv_sec: seconds as i32,
                    tv_nsec: 0,
                },
                it_value: timespec {
                    tv_sec: seconds as i32,
                    tv_nsec: 0,
                },
            };
            let mut alarm_event = MaybeUninit::<sigevent>::zeroed().as_mut_ptr();
            (*alarm_event).sigev_notify = SIGEV_SIGNAL as i32;
            (*alarm_event).sigev_signo = SIGALRM as i32;
            (*alarm_event).sigev_value.sival_ptr = *watchdog_timer;

            timer_create(CLOCK_MONOTONIC as i32, alarm_event, watchdog_timer);
            timer_settime(
                *watchdog_timer,
                0,
                &watchdog_interval,
                core::ptr::null_mut(),
            );

            Watchdog {
                timer: watchdog_timer,
                timeout: seconds,
            }
        }
    }

    pub fn reset(&self) {
        let watchdog_interval = itimerspec {
            it_interval: timespec {
                tv_sec: self.timeout as i32,
                tv_nsec: 0,
            },
            it_value: timespec {
                tv_sec: self.timeout as i32,
                tv_nsec: 0,
            },
        };
        unsafe { timer_settime(*(self.timer), 0, &watchdog_interval, core::ptr::null_mut()) };
    }
}
