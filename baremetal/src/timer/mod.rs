use super::gpio::*;

const TIMER_COUNT: usize = 2;
const GPT_BASE: usize = 0x21030000;

/// <summary>The GPT interrupts (and hence callbacks) run at this priority level.</summary>
const GPT_PRIORITY: u32 = 2;

#[derive(Copy, Clone)]
pub union TimerCallback {
    pub handler: unsafe extern "C" fn(),
    reserved: usize,
}

pub static mut TIMER_CALLBACKS: [TimerCallback; TIMER_COUNT] =
    [TimerCallback { reserved: 0 }, TimerCallback { reserved: 0 }];

struct GptInfo {
    ctrl_reg_offset: usize,
    icnt_reg_offset: usize,
}

const GPT_REG_OFFSETS: [GptInfo; TIMER_COUNT] = [
    GptInfo {
        ctrl_reg_offset: 0x10,
        icnt_reg_offset: 0x14,
    },
    GptInfo {
        ctrl_reg_offset: 0x20,
        icnt_reg_offset: 0x24,
    },
];

pub fn gpt_init() {
    // Enable INT1 in the NVIC. This allows the processor to receive an interrupt
    // from GPT0 or GPT1. The interrupt for the specific timer is enabled in Gpt_CallbackMs.

    // IO CM4 GPT0 timer and GPT1 timer interrupt both use INT1.
    set_nvic_priority(1, GPT_PRIORITY as u8);
    enable_nvic_interrupt(1);
}

pub extern "C" fn gpt_handle_irq1() {
    // GPT_ISR -> read, clear interrupts.
    let active_irqs = read_reg32(GPT_BASE, 0x00);
    write_reg32(GPT_BASE, 0x00, active_irqs);

    // Do not need to disable interrupts or timer because only used in one-shot mode.
    for gpt in 0..TIMER_COUNT {
        let mask = 1u32 << gpt;
        if (active_irqs & mask) == 0 {
            continue;
        }

        unsafe {
            (TIMER_CALLBACKS[gpt].handler)();
        }
    }
}

pub fn launch_timer_ms(gpt: usize, period_ms: u32, callback: TimerCallback) {
    unsafe {
        TIMER_CALLBACKS[gpt] = callback;
    }

    let mask = 1u32 << gpt;

    // GPTx_CTRL[0] = 0 -> disable if already enabled.
    clear_reg32(GPT_BASE, GPT_REG_OFFSETS[gpt].ctrl_reg_offset, 0x01);

    // The interrupt enable bits for both timers are in the same register. Therefore,
    // block timer ISRs to prevent an ISR from enabling a timer which is then disabled
    // because this function writes a zero to that bit in the IER register.

    let prev_base_pri = block_irqs();
    // GPT_IER[gpt] = 1 -> enable interrupt.
    set_reg32(GPT_BASE, 0x04, mask);
    restore_irqs(prev_base_pri);

    // GPTx_ICNT = delay in milliseconds (assuming 1KHz clock in GPTx_CTRL).
    // Note 1KHz is approximate - the precise value depends on the clock source,
    // but it will be 0.99kHz to 2 decimal places.
    write_reg32(GPT_BASE, GPT_REG_OFFSETS[gpt].icnt_reg_offset, period_ms);

    // GPTx_CTRL -> auto clear; 1kHz, one shot, enable timer.
    write_reg32(GPT_BASE, GPT_REG_OFFSETS[gpt].ctrl_reg_offset, 0x9);
}
